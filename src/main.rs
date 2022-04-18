//! Generally Underappreciated Algebraic Calculator
//!
//! `guac` is a minimal stack-based (RPN) calculator with a basic knowledge of algebra.

#![warn(missing_docs)]

/// Provides the `Expr` type and various methods for working with it
pub mod expr;

/// Various utilities
pub mod util;

mod mode;

use crate::expr::Expr;
use anyhow::{Context, Error};
use std::{
    fmt::Display,
    io::{self, StdinLock, StdoutLock, Write},
    ops::Deref,
};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};

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
            if let Some(n) = self.expr.to_f64() {
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
    mode: fn(&mut State<'a>, Key) -> bool,
    stdin: Keys<StdinLock<'a>>,
    stdout: RawTerminal<StdoutLock<'a>>,
}

impl<'a> State<'a> {
    fn render(&mut self) -> Result<(), Error> {
        let (_, cy) = self
            .stdout
            .cursor_pos()
            .context("couldn't get cursor pos")?;
        print!("{}{}", clear::CurrentLine, cursor::Goto(0, cy));
        for n in &self.stack {
            print!("{} ", n);
        }
        print!("{}", self.input);
        self.stdout.flush()?;
        Ok(())
    }

    fn push(&mut self) {
        let approx = if self.input.contains('.') {
            true
        } else {
            false
        };

        if let Ok(expr) = self.input.parse() {
            self.input.clear();
            self.stack.push(StackItem { approx, expr });
        }
    }

    fn apply_binary(&mut self, f: fn(Expr, Expr) -> Expr) {
        if !self.stack.is_empty() {
            self.push();
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

    fn apply_unary(&mut self, f: fn(Expr) -> Expr) {
        self.push();

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
        loop {
            let key = self.stdin.next().unwrap()?;

            if (self.mode)(self, key) {
                break;
            };

            self.render()?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let stdout = io::stdout();
    let stdout = stdout.lock().into_raw_mode()?;

    let stdin = io::stdin();
    let stdin = stdin.lock().keys();

    let mut state = State {
        stack: Vec::new(),
        input: String::new(),
        mode: State::normal,
        stdin,
        stdout,
    };

    state.start().context("couldn't start the event loop")?;

    Ok(())
}
