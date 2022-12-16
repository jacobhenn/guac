//! Generally Underappreciated Algebraic Calculator
//!
//! `guac` is a minimal stack-based (RPN) calculator with a basic knowledge of algebra.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

use crate::{
    args::{Args, SubCommand},
    config::Config,
    error::SoftError,
    expr::Expr,
    mode::{Mode, Status},
    radix::Radix,
};

use std::{
    fmt::{Display, Write},
    io::{self, BufRead, BufReader, StdoutLock, Write as _},
    mem,
    process::exit,
};

use anyhow::{bail, Context, Error};

use colored::Colorize;

use crossterm::{
    cursor,
    event::{self, Event},
    terminal::{self, ClearType},
    tty::IsTty,
    ExecutableCommand, QueueableCommand,
};

use num::{traits::Pow, BigInt, BigRational};

/// Provides the `Expr` type and various methods for working with it
pub mod expr;

/// Structures into which configuration is parsed.
pub mod config;

/// Types and functions for keeping track of and executing modes.
pub mod mode;

/// Types and functions for executing in-guac commands.
pub mod cmd;

/// Types and functions for parsing and displaying radices.
pub mod radix;

/// [`SoftError`], [`SoftResult`], and their `impl`s.
pub mod error;

mod args;

#[cfg(test)]
mod tests;

/// A way to display an expression to the screen, either exact or approximate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayMode {
    /// Display the expression exactly, using fractions.
    Exact,

    /// Display the expression approximately, using floats.
    Approx,
}

impl DisplayMode {
    /// Combine two display modes into a new one that represents the "least default" of the two
    /// passed in.
    ///
    /// - If either are [`DisplayMode::Approx`], it returns [`DisplayMode::Approx`].
    /// - Only if both are [`DisplayMode::Exact`] will it return [`DisplayMode::Exact`].
    fn combine(this: Self, that: Self) -> Self {
        if this == Self::Exact && that == Self::Exact {
            Self::Exact
        } else {
            Self::Approx
        }
    }
}

/// An expression, along with other data necessary for displaying it but not for doing math with it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackItem {
    expr: Expr<BigRational>,
    exact_str: String,
    approx_str: String,
    display_mode: DisplayMode,
    debug: bool,
    radix: Radix,
}

impl StackItem {
    /// Create a new `StackItem` containing an exact expression and cache its rendered strings.
    #[must_use]
    pub fn new(
        expr: Expr<BigRational>,
        radix: Radix,
        config: &Config,
        display_mode: DisplayMode,
        debug: bool,
    ) -> Self {
        let approx_expr = expr.clone().approx();
        let exact_str = expr.display(radix, config);
        let approx_str = approx_expr.display(radix, config);
        Self {
            expr,
            exact_str,
            approx_str,
            display_mode,
            debug,
            radix,
        }
    }

    /// Update the cached strings in the stack item.
    pub fn rerender(&mut self, config: &Config) {
        self.exact_str = self.expr.display(self.radix, config);
        self.approx_str = self.expr.clone().approx().display(self.radix, config);
    }

    /// Display the `StackItem` in its display mode using the (latex formatter)[latex::Formatter].
    pub fn display_latex(&self, config: &Config) -> String {
        match self.display_mode {
            DisplayMode::Exact => self.expr.display_latex(self.radix, config),
            DisplayMode::Approx => self.expr.clone().approx().display_latex(self.radix, config),
        }
    }
}

impl Display for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.debug {
            match self.display_mode {
                DisplayMode::Exact => return write!(f, "{:?}", self.expr),
                DisplayMode::Approx => return write!(f, "{:?}", self.expr.clone().approx()),
            }
        }

        match self.display_mode {
            DisplayMode::Exact => f.write_str(&self.exact_str),
            DisplayMode::Approx => f.write_str(&self.approx_str),
        }
    }
}

/// The global state of the calculator.
pub struct State<'a> {
    stack: Vec<StackItem>,

    /// A list of past stacks.
    history: Vec<Vec<StackItem>>,

    /// A list of stacks that have been undone.
    future: Vec<Vec<StackItem>>,

    /// The current text in the input field.
    input: String,

    /// The current text in the input field after "ᴇ".
    eex_input: Option<String>,

    /// The current text in the input field before "#".
    radix_input: Option<String>,

    /// The current local radix in the input field. If `self.radix_input` is empty or invalid, this should be `None`.
    input_radix: Option<Radix>,

    /// The soft error currently displaying on the modeline.
    err: Option<SoftError>,

    mode: Mode,

    /// The index of the selected item on the stack, or `None` if the input is selected.
    select_idx: Option<usize>,

    config: Config,

    stdout: StdoutLock<'a>,
}

impl<'a> State<'a> {
    const fn new(stdout: StdoutLock<'a>, config: Config) -> Self {
        Self {
            stack: Vec::new(),
            history: Vec::new(),
            future: Vec::new(),
            input: String::new(),
            eex_input: None,
            radix_input: None,
            input_radix: None,
            err: None,
            mode: Mode::Normal,
            select_idx: None,
            config,
            stdout,
        }
    }

    /// Return the index of the selected item, or the last item if none are selected.
    fn select_idx(&self) -> Option<usize> {
        self.select_idx.or_else(|| self.stack.len().checked_sub(1))
    }

    fn selected_item_mut(&mut self) -> Option<&mut StackItem> {
        if let Some(i) = self.select_idx {
            self.stack.get_mut(i)
        } else {
            self.stack.last_mut()
        }
    }

    #[inline]
    fn input_radix(&self) -> Radix {
        self.input_radix.unwrap_or(self.config.radix)
    }

    fn render(&mut self) -> Result<(), Error> {
        let (_, cy) = cursor::position().context("couldn't get cursor pos")?;
        self.stdout
            .queue(terminal::Clear(ClearType::CurrentLine))
            .context("couldn't clear the current line")?
            .queue(cursor::MoveTo(0, cy))
            .context("couldn't move the cursor to the start of the line")?;

        // the string which will be printed to the terminal, including formatting codes
        let mut s = String::new();
        // the apparent length of `s`, excluding formatting codes
        let mut len: usize = 0;
        // the midpoint of the selected expression, not as an index of `s`, but as an `x`
        // coordinate of a terminal cell; `None` if no expression is selected
        let mut selected_pos: Option<usize> = None;

        for i in 0..self.stack.len() {
            let stack_item = &self.stack[i];
            let expr_str = stack_item.to_string();

            // if the current expression we're looking at is selected, assign to `selected_pos`
            if Some(i) == self.select_idx {
                selected_pos = Some(len + expr_str.len() / 2);
                s.push_str(&format!("{} ", expr_str.underline()));
            } else {
                write!(&mut s, "{expr_str} ").unwrap();
            }

            len += expr_str.len() + 1;
        }

        if self.mode == Mode::Pipe {
            s.push('|');
            len += 1;
        } else if self.mode == Mode::Cmd {
            s.push(':');
            len += 1;
        }

        // the position of the `#` in the input as a terminal column
        let mut hash_pos = None;
        if let Some(radix_input) = &self.radix_input {
            s.push_str(radix_input);
            s.push('#');
            len += radix_input.len();
            hash_pos = Some(len);
            len += 1;
        }

        let input = self.input.to_string();
        len += input.len();
        s.push_str(&input);

        if let Some(eex_input) = &self.eex_input {
            len += eex_input.len() + 1;
            s.push('ᴇ');
            s.push_str(eex_input);
        }

        let width = terminal::size().context("couldn't get terminal size")?.0 as usize;

        if len > (width - 1) {
            if let Some(pos) = selected_pos {
                // we have to crop `s` *around* the selected expr
                // the total length in chars of all the formatting escape codes in `s`
                let garbage = s.len().saturating_sub(len);
                let half_width = width / 2;
                // the leftmost index of `s` which will actually be displayed on the terminal
                let left = pos.saturating_sub(half_width);
                if let Some(i) = &mut hash_pos {
                    *i = i.saturating_sub(left);
                }

                // ditto for rightmost
                let right = (left + garbage + width - 1).clamp(0, s.len());

                s = s[left..right].to_string();
            } else {
                // no selected expr, so we can just crop off the left
                s.replace_range(0..len.saturating_sub(width - 1), "");
            }
        }

        print!("{}", s);

        if self.mode == Mode::Radix {
            if let Some(i) = hash_pos {
                self.stdout
                    .queue(cursor::MoveToColumn(i as u16 + 1))
                    .context("couldn't move cursor")?;
            }
        }

        if self.select_idx.is_some() && self.mode != Mode::Pipe && self.mode != Mode::Radix {
            self.stdout
                .queue(cursor::Hide)
                .context("couldn't hide cursor")?;
        } else {
            self.stdout
                .queue(cursor::Show)
                .context("couldn't show cursor")?;
        }

        self.stdout.flush().context("couldn't flush stdout")?;

        Ok(())
    }

    fn push_expr(&mut self, expr: Expr<BigRational>, radix: Radix, display_mode: DisplayMode) {
        self.push_stack_item(StackItem::new(
            expr,
            radix,
            &self.config,
            display_mode,
            false,
        ));
    }

    fn push_stack_item(&mut self, stack_item: StackItem) {
        self.stack
            .insert(self.select_idx.unwrap_or(self.stack.len()), stack_item);

        if let Some(ref mut i) = self.select_idx {
            *i += 1;
        }
    }

    fn drop(&mut self) {
        if let Some(i) = self.select_idx {
            self.stack.remove(i);

            if i == self.stack.len() {
                self.select_idx = None;
            }
        } else {
            self.stack.pop();
        }
    }

    fn parse_exact_expr(&self, s: &str) -> Result<Expr<BigRational>, SoftError> {
        self.input_radix()
            .parse_bigint(s)
            .map(|n| Expr::Num(BigRational::from(n)))
            .ok_or(SoftError::BadInput)
    }

    fn parse_approx_expr(&self, s: &str) -> Result<Expr<BigRational>, SoftError> {
        let (int_str, frac_str) = s.split_once('.').ok_or(SoftError::BadInput)?;

        let int_part = self
            .input_radix()
            .parse_bigint(int_str)
            .ok_or(SoftError::BadInput)?;

        let frac_part = self
            .input_radix()
            .parse_bigint(frac_str)
            .ok_or(SoftError::BadInput)?;

        let denom = BigInt::from(self.input_radix().get()).pow(frac_str.len());
        Ok(Expr::Num(
            BigRational::from(int_part) + BigRational::new(frac_part, denom),
        ))
    }

    fn parse_expr(&self, s: &str) -> Result<(DisplayMode, Expr<BigRational>), SoftError> {
        if s.contains('.') {
            let e = self.parse_approx_expr(s)?;
            Ok((DisplayMode::Approx, e))
        } else {
            let e = self.parse_exact_expr(s)?;
            Ok((DisplayMode::Exact, e))
        }
    }

    fn push_input(&mut self) -> Result<Option<String>, SoftError> {
        if self.input.is_empty() {
            // pressing `enter` when the input looks like `hex#` should alter the radix of the top
            // or selected stack item
            if self.input_radix.is_some() {
                if let Some(idx) = self.select_idx() {
                    self.stack[idx].radix = self.input_radix.unwrap_or(self.config.radix);
                    self.stack[idx].rerender(&self.config);

                    self.input_radix = None;
                    self.radix_input = None;
                    self.reset_mode();
                }
            }

            return Ok(None);
        }

        let radix = self.input_radix();

        let eex = self
            .eex_input
            .as_ref()
            .map(|eex_input| radix.parse_bigint(eex_input).ok_or(SoftError::BadRadix))
            .transpose()?;

        let (display_mode, mut expr) = self.parse_expr(&self.input)?;
        if let Some(eex) = eex {
            expr *= Expr::from(radix).pow(Expr::from(eex));
        }

        self.push_expr(expr, radix, display_mode);

        let prev_input = mem::take(&mut self.input);
        self.eex_input = None;
        self.radix_input = None;
        self.input_radix = None;
        self.reset_mode();

        Ok(Some(prev_input))
    }

    fn push_var(&mut self) {
        if !self.input.is_empty() {
            let input = mem::take(&mut self.input);
            self.push_expr(Expr::Var(input), self.input_radix(), DisplayMode::Exact);
        }
    }

    #[allow(clippy::type_complexity)] // it's not *that* bad.
    fn apply_binary(
        &mut self,
        f: &dyn Fn(Expr<BigRational>, Expr<BigRational>) -> Expr<BigRational>,
        check_domain: &dyn Fn(&Expr<BigRational>, &Expr<BigRational>) -> Option<SoftError>,
    ) -> Result<(), SoftError> {
        let prev_input = if self.select_idx.is_none() {
            self.push_input()?
        } else {
            None
        };

        if self.stack.len() < 2 || self.select_idx == Some(0) {
            return Ok(());
        }

        let idx = self.select_idx().unwrap();

        if let Some(e) = check_domain(&self.stack[idx - 1].expr, &self.stack[idx].expr) {
            if let Some(prev_input) = prev_input {
                self.stack.pop();
                self.input = prev_input;
            }

            return Err(e);
        }

        // expr0 expr1 expr2 expr3
        //       ^^^^^ ^^^^^
        //       |     | y <- idx
        //       | x <- idx - 1
        let x = self.stack.remove(idx - 1);
        let y = self.stack.remove(idx - 1);

        let display_mode = DisplayMode::combine(x.display_mode, y.display_mode);

        let item = StackItem::new(
            f(x.expr, y.expr),
            x.radix,
            &self.config,
            display_mode,
            x.debug || y.debug,
        );

        // expr0 expr4 expr3
        //       ^^^^^
        //       | idx - 1
        self.stack.insert(idx - 1, item);

        if let Some(ref mut i) = self.select_idx {
            *i -= 1;
        }

        Ok(())
    }

    fn apply_unary(
        &mut self,
        f: &dyn Fn(Expr<BigRational>) -> Expr<BigRational>,
        check_domain: &dyn Fn(&Expr<BigRational>) -> Option<SoftError>,
    ) -> Result<(), SoftError> {
        let prev_input = if self.select_idx.is_none() {
            self.push_input()?
        } else {
            None
        };

        if self.stack.is_empty() {
            return Ok(());
        }

        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);

        if let Some(e) = check_domain(&self.stack[idx].expr) {
            if let Some(prev_input) = prev_input {
                self.stack.pop();
                self.input = prev_input;
            }

            return Err(e);
        }

        let x = self.stack.remove(idx);
        let item = StackItem::new(f(x.expr), x.radix, &self.config, x.display_mode, x.debug);
        self.stack.insert(idx, item);

        Ok(())
    }

    fn dup(&mut self) {
        if !self.stack.is_empty() {
            let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
            let e = self.stack[idx].clone();
            self.stack.insert(idx + 1, e);
            if let Some(i) = self.select_idx.as_mut() {
                *i += 1;
            }
        }
    }

    fn swap(&mut self) {
        let Some(idx) = self.select_idx() else { return; };
        if idx > 0 {
            self.stack.swap(idx - 1, idx);
        }
    }

    fn toggle_approx(&mut self) {
        let Some(item) = self.selected_item_mut() else { return; };
        match &mut item.display_mode {
            m @ DisplayMode::Approx => *m = DisplayMode::Exact,
            m @ DisplayMode::Exact => *m = DisplayMode::Approx,
        }
    }

    fn toggle_debug(&mut self) {
        let Some(item) = self.selected_item_mut() else { return; };
        item.debug = !item.debug;
    }

    fn init_from_stdin(&mut self) {
        let stdin = io::stdin();

        if stdin.is_tty() {
            return;
        }

        let stdin = BufReader::new(stdin);
        let mut lines = stdin.lines();
        let mut idx: usize = 0;
        let mut bad_idxs = Vec::new();
        while let Some(Ok(line)) = lines.next() {
            idx += 1;
            let line: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            if let Ok((m, e)) = self.parse_expr(&line) {
                self.push_expr(e, self.config.radix, m);
            } else {
                bad_idxs.push(idx);
            }
        }

        if !bad_idxs.is_empty() {
            eprintln!(
                "{}{} couldn't parse stdin (line{} {})",
                "info".bold().cyan(),
                ":".bold(),
                if bad_idxs.len() == 1 { "" } else { "s" },
                bad_idxs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    fn ev_loop(&mut self) -> Result<(), Error> {
        loop {
            self.err = None;

            // Read the next event from the terminal.
            if let Event::Key(kev) = event::read().context("couldn't get next terminal event")? {
                match self.handle_keypress(kev) {
                    Ok(Status::Render) => {
                        self.write_modeline().context("couldn't write modeline")?;
                        self.render().context("couldn't render the state")?;
                        if let Some(old_stack) = self.history.last() {
                            if &self.stack != old_stack {
                                self.future = Vec::new();
                                self.history.push(self.stack.clone());
                            }
                        } else {
                            self.future = Vec::new();
                            self.history.push(self.stack.clone());
                        }
                    }
                    Ok(Status::Exit) => {
                        break;
                    }
                    Ok(Status::Undo) => {
                        if self.future.is_empty() {
                            self.history.pop();
                        }

                        if let Some(mut old_stack) = self.history.pop() {
                            mem::swap(&mut old_stack, &mut self.stack);
                            self.future.push(old_stack);
                        }

                        self.render().context("couldn't render the state")?;
                    }
                    Ok(Status::Redo) => {
                        if let Some(mut new_stack) = self.future.pop() {
                            mem::swap(&mut new_stack, &mut self.stack);
                            self.history.push(new_stack);
                        }
                        self.render().context("couldn't render the state")?;
                    }
                    #[cfg(debug_assertions)]
                    Ok(Status::Debug) => bail!("debug"),
                    Err(e) => {
                        self.err = Some(e);
                        // TODO: decide if we really need to render the whole stack here
                        self.write_modeline().context("couldn't write modeline")?;
                        self.render().context("couldn't render the state")?;
                    }
                }
            }
        }

        Ok(())
    }

    fn start(&mut self) -> Result<(), Error> {
        terminal::enable_raw_mode().context("couldn't enable raw mode")?;

        let (cx, cy) = cursor::position().context("couldn't get cursor position")?;
        let (.., height) = terminal::size().context("couldn't get terminal size")?;

        // If the cursor is at the bottom of the screen, make room for one more line.
        if cy >= height - 1 {
            println!();
            self.stdout
                .execute(cursor::MoveTo(cx, cy - 1))
                .context("couldn't move cursor")?;
        }

        self.write_modeline().context("couldn't write modeline")?;
        self.render().context("couldn't render the state")?;

        self.ev_loop()?;

        Ok(())
    }
}

#[allow(unused_must_use)]
/// Try our best to clean up the terminal state; if too many errors happen, just print some
/// newlines and call it good.
fn cleanup() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    if stdout.is_tty() {
        stdout.execute(cursor::Show);
        if terminal::disable_raw_mode().is_ok() {
            println!();
        } else {
            print!("\n\r\n\r");
        }
        stdout.execute(terminal::Clear(ClearType::CurrentLine));
    }
}

fn guac_interactive(force: bool) -> Result<(), Error> {
    let stdout = io::stdout();
    let stdout = stdout.lock();

    if !force {
        if !stdout.is_tty() {
            bail!("stdout is not a tty. use --force to run anyway.");
        } else if terminal::size().context("couldn't get terminal size")?.0 < 15 {
            bail!("terminal is too small. use --force to run anyway.")
        }
    }

    let config = Config::default();
    let mut state = State::new(stdout, config);

    state.init_from_stdin();

    state.start()?;

    Ok(())
}

fn go() -> Result<(), Error> {
    let args: Args = argh::from_env();

    match args.subc {
        Some(SubCommand::Keys(..)) => print!(include_str!("keys.txt")),
        Some(SubCommand::Version(..)) => {
            println!("guac v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            guac_interactive(args.force)?;
            cleanup();
        }
    }

    Ok(())
}

fn main() {
    let res = go();
    if let Err(e) = res {
        eprintln!("{}{} {e:#}", "guac error".bold().red(), ":".bold());
        exit(1);
    }
}
