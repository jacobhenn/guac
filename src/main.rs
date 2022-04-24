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
use config::Config;
use crossterm::{
    cursor,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use num::traits::Pow;
use std::{
    fmt::Display,
    io::{stdout, StdoutLock, Write},
    ops::Deref,
};

const RADIX: u32 = 10;

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
    err: String,
    mode: fn(&mut State<'a>) -> Result<bool, Error>,
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
        for n in &self.stack {
            s.push_str(&format!("{} ", n));
        }

        s.push_str(&self.input.to_string());

        let (width, ..) = terminal::size().context("couldn't get terminal size")?;
        let width = width - 1;

        if s.len() > width as usize {
            s.replace_range(0..s.len().saturating_sub(width as usize), "");
        }

        print!("{}", s);
        self.stdout.flush()?;

        Ok(())
    }

    fn push_expr(&mut self, expr: Expr) {
        self.stack.push(StackItem {
            approx: false,
            expr,
        });
    }

    fn push_input(&mut self) {
        let input = self.input.parse::<Expr>();
        if let Ok(expr) = input {
            self.input.clear();
            if let Ok(eex) = self.eex_input.parse::<i128>() {
                self.eex_input.clear();
                self.stack.push(StackItem {
                    approx: self.input.contains('.') || eex.is_negative(),
                    expr: expr * Expr::from_int(RADIX).pow(eex.into()),
                });
            } else {
                self.stack.push(StackItem {
                    approx: self.input.contains('.'),
                    expr,
                });
            }
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

    fn apply_binary<F>(&mut self, f: F)
    where
        F: Fn(Expr, Expr) -> Expr,
    {
        if !self.stack.is_empty() {
            self.push_input();
        }

        if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(StackItem {
                    approx: x.approx || y.approx,
                    expr: f((*y).clone(), (*x).clone()),
                });
            }
        }
    }

    fn apply_unary<F>(&mut self, f: F)
    where
        F: Fn(Expr) -> Expr,
    {
        self.push_input();

        if !self.stack.is_empty() {
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
        if cy == height {
            println!();
            self.stdout.execute(cursor::MoveTo(cx, cy - 1))?;
        }

        loop {
            if (self.mode)(self).context("couldn't tick state")? {
                break;
            };
        }

        println!();

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
        err: String::new(),
        mode: State::normal,
        config: Config::default(),
        stdout,
    };

    state.start().context("couldn't start the event loop")?;

    Ok(())
}
