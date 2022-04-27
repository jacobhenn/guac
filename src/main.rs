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

mod config;

mod mode;

use crate::expr::Expr;
use anyhow::{Context, Error};
use colored::Colorize;
use config::Config;
use crossterm::{
    cursor, event,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use mode::Mode;
use num::{traits::Pow, BigInt, BigRational, Signed};
use std::{
    fmt::Display,
    io::{stdout, StdoutLock, Write},
    ops::Deref,
};

const RADIX: u32 = 10;

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
        }
    }
}

/// An expression, along with other data necessary for displaying it but not for doing math with it.
#[derive(Clone)]
pub struct StackItem {
    approx: bool,
    expr: Expr,
}

impl Display for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.approx {
            if let Ok(n) = self.expr.clone().to_f64() {
                return write!(f, "{:.3}", n);
            }
        }
        write!(f, "{}", self.expr)
    }
}

impl Deref for StackItem {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.expr
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
            .queue(terminal::Clear(ClearType::CurrentLine))?
            .queue(cursor::MoveTo(0, cy))?;

        let mut s = String::new();
        for i in 0..self.stack.len() {
            if Some(i) == self.select_idx {
                s.push_str(&format!("{} ", self.stack[i].to_string().underline()));
            } else {
                s.push_str(&format!("{} ", self.stack[i]));
            }
        }

        s.push_str(&self.input.to_string());
        if self.eex {
            s.push_str(&format!("e{}", self.eex_input));
        }

        let (width, ..) = terminal::size().context("couldn't get terminal size")?;
        let width = width - 1;

        if s.len() > width as usize {
            s.replace_range(0..s.len().saturating_sub(width as usize), "");
        }

        print!("{}", s);

        if self.select_idx.is_some() {
            self.stdout.queue(cursor::Hide)?;
        } else {
            self.stdout.queue(cursor::Show)?;
        }

        self.stdout.flush()?;

        Ok(())
    }

    fn push_expr(&mut self, expr: Expr) {
        self.stack.push(StackItem {
            approx: false,
            expr,
        });
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
                self.stack.push(StackItem {
                    approx: self.input.contains('.') || eex.is_negative(),
                    expr: expr * Expr::from_int(RADIX).pow(Expr::Num(BigRational::from(eex))),
                });
                self.eex = false;
                true
            } else if self.eex {
                self.err = Some(SoftError::BadEex);
                false
            } else {
                self.input.clear();
                self.stack.push(StackItem {
                    approx: self.input.contains('.'),
                    expr,
                });
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
            self.stack.push(StackItem {
                approx: false,
                expr: Expr::Var(self.input.clone()),
            });
            self.input.clear();
        }
    }

    fn apply_binary<F, G>(&mut self, f: F, are_in_domain: G)
    where
        F: Fn(Expr, Expr) -> Expr,
        G: Fn(&Expr, &Expr) -> Option<SoftError>,
    {
        let pushed_input = if self.stack.is_empty() {
            false
        } else {
            self.push_input()
        };

        if self.stack.len() >= 2 {
            if let Some(e) = are_in_domain(
                &self.stack[self.stack.len() - 2],
                self.stack.last().unwrap(),
            ) {
                self.err = Some(e);

                if pushed_input {
                    self.input = self.stack.pop().unwrap().to_string();
                }

                return;
            }

            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(StackItem {
                    approx: x.approx || y.approx,
                    expr: f((*y).clone(), (*x).clone()),
                });
            }
        }
    }

    fn apply_unary<F, G>(&mut self, f: F, is_in_domain: G)
    where
        F: Fn(Expr) -> Expr,
        G: Fn(&Expr) -> Option<SoftError>,
    {
        let pushed_input = self.push_input();

        if !self.stack.is_empty() {
            if let Some(e) = is_in_domain(self.stack.last().unwrap()) {
                self.err = Some(e);

                if pushed_input {
                    self.input = self.stack.pop().unwrap().to_string();
                }

                return;
            }

            if let Some(x) = self.stack.pop() {
                self.stack.push(StackItem {
                    approx: x.approx,
                    expr: f((*x).clone()),
                });
            }
        }
    }

    fn dup(&mut self) {
        if let Some(l) = self.stack.pop() {
            self.stack.push(l.clone());
            self.stack.push(l);
        }
    }

    fn swap(&mut self) {
        if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(x);
                self.stack.push(y);
            }
        }
    }

    fn toggle_approx(&mut self) {
        if let Some(x) = self.stack.last_mut() {
            x.approx = !x.approx;
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let (cx, cy) = cursor::position().context("couldn't get cursor pos")?;
        let (.., height) = terminal::size().context("couldn't get terminal size")?;

        // If the cursor is at the bottom of the screen, make room for one more line.
        if cy >= height - 1 {
            println!();
            self.stdout.execute(cursor::MoveTo(cx, cy - 1))?;
        }

        loop {
            self.write_modeline().context("couldn't write modeline")?;

            match event::read()? {
                event::Event::Key(k) => {
                    if match self.mode {
                        Mode::Normal => self.normal(k),
                        Mode::Constant => self.constant(k),
                        Mode::MassConstant => self.mass_constant(k),
                        Mode::Variable => self.variable(k),
                    }? {
                        break;
                    }
                }
                event::Event::Resize(_, _) => {
                    self.render().context("couldn't render")?;
                    self.write_modeline().context("couldn't write modeline")?;
                }
                _ => (),
            }
        }

        self.stdout
            .execute(cursor::Show)
            .context("while cleaning up: couldn't show cursor")?;
        terminal::disable_raw_mode().context("while cleaning up: couldn't disable raw mode")?;

        println!();
        self.stdout
            .execute(terminal::Clear(ClearType::CurrentLine))
            .context("while cleaning up: couldn't clear modeline")?;

        if self.stack.is_empty() {
            self.stdout
                .execute(cursor::MoveUp(1))
                .context("while cleaning up: couldn't move cursor")?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let stdout = stdout();
    let stdout = stdout.lock();

    terminal::enable_raw_mode()?;

    let mut state = State {
        stack: Vec::new(),
        input: String::new(),
        eex_input: String::new(),
        eex: false,
        err: None,
        mode: Mode::Normal,
        select_idx: None,
        config: Config::default(),
        stdout,
    };

    state.start().context("couldn't start the event loop")?;

    Ok(())
}
