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
#![allow(clippy::must_use_candidate)]

use crate::{
    args::{Args, SubCommand},
    config::Config,
    expr::Expr,
    mode::{Mode, Status},
    radix::Radix,
};

use std::{
    fmt::Display,
    io::{self, stdin, stdout, BufRead, BufReader, ErrorKind, StdoutLock, Write},
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

use num::{traits::Pow, BigRational};

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

mod args;

#[cfg(test)]
mod tests;

/// A representation of an error on the user's end.
pub enum SoftError {
    /// Operation would divided by zero.
    DivideByZero,

    /// Operation would produce a complex result, which is not yet supported by `guac`.
    Complex,

    /// Input could not be parsed.
    BadInput,

    /// Eex input (input after the `e` in e-notation) could not be parsed.
    BadEex,

    /// Radix input (input before the `#` in `guac` radix notation) could not be parsed.
    BadRadix,

    /// The argument of `tan` was not in its domain.
    BadTan,

    /// The argument of `log` was not in its domain.
    BadLog,

    /// The command entered in pipe mode could not be run; it returned this IO error.
    BadSysCmd(io::Error),

    /// The command entered in pipe mode failed. The first arg is the name of the command. If it printed to stderr, the second arg contains the first line. If not, it is the `ExitStatus` it returned.
    SysCmdFailed(String, String),

    /// The command entered in pipe mode spawned successfully, but an IO error occurred while attempting to manipulate it.
    SysCmdIoErr(anyhow::Error),

    /// The command entered in command mode was not recognized.
    UnknownGuacCmd(String),

    /// The command entered in command mode was missing an argument.
    GuacCmdMissingArg,

    /// The command entered in command mode had too many arguments.
    GuacCmdExtraArg,

    /// The path provided to the `set` command was bad.
    BadSetPath(String),

    /// The value provided to the `set` command could not be parsed.
    BadSetVal(String),

    /// This error should never be thrown in a release. It's just used to debug certain things.
    #[cfg(debug_assertions)]
    Debug(String),
}

impl Display for SoftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "E00: divide by zero"),
            Self::Complex => write!(f, "E01: complex not yet supported"),
            Self::BadInput => write!(f, "E02: bad input"),
            Self::BadEex => write!(f, "E03: bad eex input"),
            Self::BadRadix => write!(f, "E04: bad radix"),
            Self::BadTan => write!(f, "E05: tangent of π/2"),
            Self::BadLog => write!(f, "E06: log of n ≤ 0"),
            Self::BadSysCmd(e) => {
                if e.kind() == ErrorKind::NotFound {
                    write!(f, "E07: unknown command")
                } else {
                    write!(f, "E08: bad command: {e}")
                }
            }
            Self::SysCmdFailed(s, e) => write!(f, "E09: {s}: {e}"),
            Self::SysCmdIoErr(e) => write!(f, "E10: cmd io err: {e}"),
            Self::UnknownGuacCmd(s) => write!(f, "E11: unknown cmd {s}"),
            Self::GuacCmdMissingArg => write!(f, "E12: cmd missing arg"),
            Self::GuacCmdExtraArg => write!(f, "E13: too many cmd args"),
            Self::BadSetPath(p) => write!(f, "E14: no such setting \"{p}\"",),
            Self::BadSetVal(v) => write!(f, "E15: couldnt parse \"{v}\"",),
            #[cfg(debug_assertions)]
            Self::Debug(s) => write!(f, "DEBUG: {s}"),
        }
    }
}

/// An expression, along with other data necessary for displaying it but not for doing math with it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackItem {
    expr: Expr<BigRational>,
    radix: Radix,
    approx: bool,
    exact_str: String,
    approx_str: String,
}

impl StackItem {
    /// Create a new `StackItem` and cache its rendered strings.
    pub fn new(approx: bool, expr: &Expr<BigRational>, radix: Radix, config: &Config) -> Self {
        Self {
            expr: expr.clone(),
            radix,
            approx,
            exact_str: expr.display(radix, config),
            approx_str: expr.clone().approx().display(radix, config),
        }
    }

    /// Update the cached strings in a stack item.
    pub fn rerender(&mut self, config: &Config) {
        self.exact_str = self.expr.display(self.radix, config);
        self.approx_str = self.expr.clone().approx().display(self.radix, config);
    }
}

impl Display for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.approx {
            write!(f, "{}", self.approx_str)
        } else {
            write!(f, "{}", self.exact_str)
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
    fn selected_or_last_idx(&self) -> Option<usize> {
        self.select_idx
            .map_or_else(|| self.stack.len().checked_sub(1), Some)
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
        // the midpoint of the selected expression, not as an index of `s`, but as an `x` coordinate of a terminal cell; `None` if no expression is selected
        let mut selected_pos = None;

        for i in 0..self.stack.len() {
            let stack_item = &self.stack[i];

            let expr_str = if option_env!("GUAC_DEBUG") == Some("true") {
                format!("{:?}", stack_item.expr)
            } else {
                stack_item.to_string()
            };

            // if the current expression we're looking at is selected, assign to `selected_pos`
            if Some(i) == self.select_idx {
                selected_pos = Some(len + expr_str.len() / 2);
                s.push_str(&format!("{} ", expr_str.underline()));
            } else {
                s.push_str(&format!("{expr_str} "));
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

    fn push_expr(&mut self, expr: &Expr<BigRational>, radix: Radix) {
        self.stack.insert(
            self.select_idx.unwrap_or(self.stack.len()),
            StackItem::new(false, expr, radix, &self.config),
        );

        if let Some(i) = &mut self.select_idx {
            *i += 1;
        }
    }

    fn push_stack_item(&mut self, stack_item: StackItem) {
        self.stack
            .insert(self.select_idx.unwrap_or(self.stack.len()), stack_item);

        if let Some(i) = &mut self.select_idx {
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

    // TODO: should this return an Expr<f64> if it can't parse an int?
    fn parse_expr(&self, s: &str) -> Result<Expr<BigRational>, SoftError> {
        let radix = self.input_radix.unwrap_or(self.config.radix);

        if let Some(int) = radix.parse_bigint(s) {
            return Ok(Expr::Num(BigRational::from(int)));
        } else if radix == self.config.radix {
            if let Ok(n) = s.parse::<f64>() {
                if let Some(n) = BigRational::from_float(n) {
                    return Ok(Expr::Num(n));
                }
            }
        }

        Err(SoftError::BadInput)
    }

    fn push_input(&mut self) -> bool {
        if self.input.is_empty() {
            if self.input_radix.is_none() {
                return false;
            } else if let Some(i) = self.selected_or_last_idx() {
                if let Some(x) = self.stack.get_mut(i) {
                    x.radix = self.input_radix.unwrap_or(self.config.radix);
                    x.rerender(&self.config);
                }

                self.input_radix = None;
                self.radix_input = None;
                self.reset_mode();

                return false;
            }
        }

        let mut expr = match self.parse_expr(&self.input) {
            Ok(e) => e,
            Err(e) => {
                self.err = Some(e);
                return false;
            }
        };

        let radix = self.input_radix.unwrap_or(self.config.radix);
        if let Some(eex_input) = &self.eex_input {
            if let Some(int) = radix.parse_bigint(eex_input) {
                let exponent = Expr::Num(BigRational::from(int));
                let factor = Expr::from(radix).pow(exponent);
                expr *= factor;
            } else {
                self.err = Some(SoftError::BadEex);
                return false;
            }
        }

        let stack_item = StackItem::new(
            self.input.contains('.')
                || self
                    .eex_input
                    .as_ref()
                    .map(|s| s.contains('-'))
                    .unwrap_or_default(),
            &expr,
            radix,
            &self.config,
        );

        self.push_stack_item(stack_item);
        self.input = String::new();
        self.eex_input = None;
        self.radix_input = None;
        self.input_radix = None;
        self.reset_mode();

        true
    }

    fn push_var(&mut self) {
        if !self.input.is_empty() {
            self.stack.push(StackItem::new(
                false,
                &Expr::Var(self.input.clone()),
                self.input_radix.unwrap_or(self.config.radix),
                &self.config,
            ));
            self.input.clear();
        }
    }

    fn apply_binary<F, G>(&mut self, f: F, are_in_domain: G)
    where
        F: Fn(Expr<BigRational>, Expr<BigRational>) -> Expr<BigRational>,
        G: Fn(&Expr<BigRational>, &Expr<BigRational>) -> Option<SoftError>,
    {
        let did_push_input = if self.stack.is_empty() || self.select_idx.is_some() {
            false
        } else {
            self.push_input()
        };

        if self.stack.len() >= 2 && self.select_idx.map_or(true, |i| i > 0) {
            let idx = self
                .select_idx
                .unwrap_or_else(|| self.stack.len().saturating_sub(1));

            if let Some(e) = are_in_domain(&self.stack[idx - 1].expr, &self.stack[idx].expr) {
                self.err = Some(e);

                if did_push_input {
                    self.input = self.stack.pop().unwrap().to_string();
                }
            } else {
                let x = self.stack.remove(idx - 1);
                let y = self.stack.remove(idx - 1);

                self.stack.insert(
                    idx - 1,
                    StackItem::new(
                        x.approx || y.approx,
                        &f(x.expr, y.expr),
                        x.radix,
                        &self.config,
                    ),
                );

                if let Some(i) = self.select_idx.as_mut() {
                    *i -= 1;
                }
            }
        }
    }

    fn apply_unary<F, G>(&mut self, f: F, is_in_domain: G)
    where
        F: Fn(Expr<BigRational>) -> Expr<BigRational>,
        G: Fn(&Expr<BigRational>) -> Option<SoftError>,
    {
        let did_push_input = if self.select_idx.is_some() {
            false
        } else {
            self.push_input()
        };

        if !self.stack.is_empty() {
            let idx = self.select_idx.unwrap_or(self.stack.len() - 1);

            if let Some(e) = is_in_domain(&self.stack[idx].expr) {
                self.err = Some(e);

                if did_push_input {
                    self.input = self.stack.pop().unwrap().to_string();
                }
            } else {
                let x = self.stack.remove(idx);
                self.stack.insert(
                    idx,
                    StackItem::new(x.approx, &f(x.expr), x.radix, &self.config),
                );
            }
        }
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
        if let Some(idx) = self.selected_or_last_idx() {
            if idx > 0 {
                self.stack.swap(idx - 1, idx);
            }
        }
    }

    fn toggle_approx(&mut self) {
        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
        if let Some(x) = self.stack.get_mut(idx) {
            x.approx = !x.approx;
        }
    }

    fn init_from_stdin(&mut self) {
        let stdin = stdin();

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
            if let Ok(e) = self.parse_expr(&line) {
                self.push_expr(&e, self.config.radix);
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
                    Status::Render => {
                        self.write_modeline().context("couldn't write modeline")?;
                        self.render().context("couldn't render the state")?;
                        if let Some(old_stack) = self.history.last() {
                            if &self.stack != old_stack {
                                self.future.clear();
                                self.history.push(self.stack.clone());
                            }
                        } else {
                            self.future.clear();
                            self.history.push(self.stack.clone());
                        }
                    }
                    Status::Exit => {
                        break;
                    }
                    Status::Undo => {
                        if self.future.is_empty() {
                            self.history.pop();
                        }

                        if let Some(mut old_stack) = self.history.pop() {
                            mem::swap(&mut old_stack, &mut self.stack);
                            self.future.push(old_stack);
                        }

                        self.render().context("couldn't render the state")?;
                    }
                    Status::Redo => {
                        if let Some(mut new_stack) = self.future.pop() {
                            mem::swap(&mut new_stack, &mut self.stack);
                            self.history.push(new_stack);
                        }
                        self.render().context("couldn't render the state")?;
                    }
                    #[cfg(debug_assertions)]
                    Status::Debug => bail!("debug"),
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
/// Try our best to clean up the terminal state; if too many errors happen, just print some newlines and call it good.
fn cleanup() {
    let stdout = stdout();
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
    let stdout = stdout();
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
