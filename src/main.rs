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

/// Provides the `Expr` type and various methods for working with it
pub mod expr;

mod args;

/// Structures into which configuration is parsed.
pub mod config;

/// Types and functions for keeping track of and executing modes.
pub mod mode;

#[cfg(test)]
mod tests;

use crate::args::Args;
use crate::expr::Expr;
use anyhow::{bail, Context, Error};
use args::SubCommand;
use colored::Colorize;
use config::Config;
use crossterm::{
    cursor,
    event::{self, Event},
    terminal::{self, ClearType},
    tty::IsTty,
    ExecutableCommand, QueueableCommand,
};
use mode::{Mode, Status};
use num::{traits::Pow, BigInt, BigRational, Signed};
use std::{
    fmt::Display,
    io::{self, stdin, stdout, BufRead, BufReader, ErrorKind, StdoutLock, Write},
};

const RADIX: u32 = 10;

const RADIX_POW_SIX: u32 = 1_000_000;

const RADIX_POW_FOUR_INV: f64 = 0.001;

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

    /// The argument of `tan` was not in its domain.
    BadTan,

    /// The argument of `log` was not in its domain.
    BadLog,

    /// The command entered in pipe mode could not be run; it returned this IO error.
    BadCmd(io::Error),

    /// The command entered in pipe mode failed. The first arg is the name of the command. If it printed to stderr, the second arg contains the first line. If not, it is the `ExitStatus` it returned.
    CmdFailed(String, String),

    /// The command entered in pipe mode spawned successfully, but an IO error occurred while attempting to manipulate it.
    CmdIoErr(anyhow::Error),

    /// This error should never be thrown in a release. It's just used to debug certain things.
    #[cfg(debug_assertions)]
    Debug(String),
}

impl Display for SoftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "E0: divide by zero"),
            Self::Complex => write!(f, "E1: complex not yet supported"),
            Self::BadInput => write!(f, "E2: bad input"),
            Self::BadEex => write!(f, "E3: bad eex input"),
            Self::BadTan => write!(f, "E4: tangent of π/2"),
            Self::BadLog => write!(f, "E5: log of n ≤ 0"),
            Self::BadCmd(e) => {
                if e.kind() == ErrorKind::NotFound {
                    write!(f, "E6: unknown command")
                } else {
                    write!(f, "E7: bad command: {e}")
                }
            }
            Self::CmdFailed(s, e) => write!(f, "E8: {s}: {e}"),
            Self::CmdIoErr(e) => write!(f, "E9: cmd io err: {e}"),
            #[cfg(debug_assertions)]
            Self::Debug(s) => write!(f, "DEBUG: {s}"),
        }
    }
}

/// An expression, along with other data necessary for displaying it but not for doing math with it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackItem {
    approx: bool,
    expr: Expr,
    exact_str: String,
    approx_str: String,
}

impl StackItem {
    /// Create a new `StackItem` and cache its rendered strings.
    pub fn new(approx: bool, expr: Expr) -> Self {
        Self {
            approx,
            expr: expr.clone(),
            exact_str: expr.clone().to_string(),
            approx_str: expr.display_approx(),
        }
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
    input: String,
    eex_input: String,
    eex: bool,
    err: Option<SoftError>,
    mode: Mode,
    select_idx: Option<usize>,
    config: Config,
    stdout: StdoutLock<'a>,
}

impl<'a> State<'a> {
    fn render(&mut self) -> Result<(), Error> {
        let (_, cy) = cursor::position().context("couldn't get cursor pos")?;
        self.stdout
            .queue(terminal::Clear(ClearType::CurrentLine))
            .context("couldn't clear the current line")?
            .queue(cursor::MoveTo(0, cy))
            .context("couldn't move the cursor to the start of the line")?;

        let mut s = String::new();
        let mut len: usize = 0;
        let mut selected_pos = None;
        for i in 0..self.stack.len() {
            let stack_item = &mut self.stack[i];

            let expr_str = if option_env!("GUAC_DEBUG") == Some("true") {
                format!("{:?}", stack_item.expr)
            } else {
                stack_item.to_string()
            };

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
        }

        let input = self.input.to_string();
        len += input.len();
        s.push_str(&input);

        if self.eex {
            let eex_input = self.eex_input.to_string();
            len += eex_input.len() + 1;
            s.push('ᴇ');
            s.push_str(&eex_input);
        }

        let width = terminal::size().context("couldn't get terminal size")?.0 as usize;

        if len > (width - 1) {
            if let Some(pos) = selected_pos {
                let garbage = s.len().saturating_sub(len);
                let half_width = width / 2;
                let left = pos.saturating_sub(half_width);
                let right = (left + garbage + width - 1).clamp(0, s.len());
                s = s[left..right].to_string();
            } else {
                s.replace_range(0..len.saturating_sub(width - 1), "");
            }
        }

        print!("{}", s);

        if self.select_idx.is_some() && self.mode != Mode::Pipe {
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

    fn push_expr(&mut self, expr: Expr) {
        self.stack.insert(
            self.select_idx.unwrap_or(self.stack.len()),
            StackItem::new(false, expr),
        );
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

    fn push_input(&mut self) -> bool {
        let input = self.input.parse::<Expr>();
        if let Ok(expr) = input {
            if let Ok(eex) = self.eex_input.parse::<BigInt>() {
                self.input.clear();
                self.eex_input.clear();
                self.stack.push(StackItem::new(
                    self.input.contains('.') || eex.is_negative(),
                    expr * Expr::from_int(RADIX).pow(Expr::Num(BigRational::from(eex))),
                ));
                self.eex = false;
                true
            } else if self.eex {
                self.err = Some(SoftError::BadEex);
                false
            } else {
                self.input.clear();
                self.stack
                    .push(StackItem::new(self.input.contains('.'), expr));
                true
            }
        } else if !self.input.is_empty() || !self.eex_input.is_empty() {
            self.err = Some(SoftError::BadInput);
            false
        } else {
            false
        }
    }

    fn push_var(&mut self) {
        if !self.input.is_empty() {
            self.stack
                .push(StackItem::new(false, Expr::Var(self.input.clone())));
            self.input.clear();
        }
    }

    fn apply_binary<F, G>(&mut self, f: F, are_in_domain: G)
    where
        F: Fn(Expr, Expr) -> Expr,
        G: Fn(&Expr, &Expr) -> Option<SoftError>,
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
                    StackItem::new(x.approx || y.approx, f(x.expr, y.expr)),
                );

                if let Some(i) = self.select_idx.as_mut() {
                    *i -= 1;
                }
            }
        }
    }

    fn apply_unary<F, G>(&mut self, f: F, is_in_domain: G)
    where
        F: Fn(Expr) -> Expr,
        G: Fn(&Expr) -> Option<SoftError>,
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
                self.stack.insert(idx, StackItem::new(x.approx, f(x.expr)));
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
        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
        if idx > 0 {
            self.stack.swap(idx - 1, idx);
        }
    }

    fn toggle_approx(&mut self) {
        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
        if let Some(x) = self.stack.get_mut(idx) {
            x.approx = !x.approx;
        }
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.stdout
            .execute(cursor::Show)
            .context("couldn't show cursor")?;
        terminal::disable_raw_mode().context("couldn't disable raw mode")?;

        println!();
        self.stdout
            .execute(terminal::Clear(ClearType::CurrentLine))
            .context("couldn't clear modeline")?;

        if self.stack.is_empty() && self.input.is_empty() {
            self.stdout
                .execute(cursor::MoveUp(1))
                .context("couldn't move cursor")?;
        }

        Ok(())
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
            if let Ok(e) = line.parse::<Expr>() {
                self.push_expr(e);
            } else {
                bad_idxs.push(idx);
            }
        }

        if !bad_idxs.is_empty() {
            eprintln!(
                "{} couldn't parse stdin (line{} {})",
                "info:".cyan().bold(),
                if bad_idxs.len() == 1 { "" } else { "s" },
                bad_idxs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    /// One tick of the event loop. Returns `Ok(true)` if the user exited.
    fn tick(&mut self) -> Result<bool, Error> {
        self.err = None;

        // Read the next event from the terminal.
        if let Event::Key(kev) = event::read().context("couldn't get next terminal event")? {
            match self.handle_keypress(kev) {
                Status::Render => {
                    self.write_modeline().context("couldn't write modeline")?;
                    self.render().context("couldn't render the state")?;
                }
                Status::Exit => {
                    return Ok(true);
                }
                Status::Debug => {
                    bail!("debugging err handling");
                }
            }
        }

        Ok(false)
    }

    fn start(&mut self) -> Result<(), Error> {
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

        match loop {
            match self.tick() {
                Ok(false) => (),
                Ok(true) => break Ok(()),
                Err(e) => break Err(e),
            }
        } {
            Ok(_) => {
                self.cleanup()
                    .context("couldn't clean up after event loop")?;
            }
            Err(e) => {
                if self.cleanup().is_err() {
                    println!("\n\r\n\r");
                }

                eprintln!("{}{} {e:#}", "error".bold().red(), ":".bold());
            }
        }

        Ok(())
    }
}

fn guac_interactive(anyway: bool) -> Result<(), Error> {
    let stdout = stdout();
    let stdout = stdout.lock();

    if !anyway && !stdout.is_tty() {
        bail!("stdout is not a tty. use `guac anyway` to run anyway.");
    }

    terminal::enable_raw_mode().context("couldn't enable raw mode")?;

    let mut state = State {
        stack: Vec::new(),
        // history: Vec::new(),
        // future: Vec::new(),
        input: String::new(),
        eex_input: String::new(),
        eex: false,
        err: None,
        mode: Mode::Normal,
        select_idx: None,
        config: Config::default(),
        stdout,
    };

    state.init_from_stdin();

    state.start().context("couldn't start the event loop")?;

    Ok(())
}

fn go() -> Result<(), Error> {
    let args: Args = argh::from_env();

    match args.subc {
        Some(SubCommand::Keys(..)) => print!(include_str!("keys.txt")),
        Some(SubCommand::Anyway(..)) => guac_interactive(true)?,
        None => {
            if terminal::size().context("couldn't get terminal size")?.0 < 15 {
                bail!("terminal is too small. use `guac anyway` to run anyway.")
            }

            guac_interactive(false)?;
        }
    }

    Ok(())
}

fn main() {
    match go() {
        Ok(_) => (),
        Err(e) => {
            if terminal::disable_raw_mode().is_err() {
                println!("\n\r\n\r");
            }

            eprintln!("{}{} {e:#}", "error".bold().red(), ":".bold());
        }
    }
}
