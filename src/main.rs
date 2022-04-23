//! Generally Underappreciated Algebraic Calculator
//!
//! `guac` is a minimal stack-based (RPN) calculator with a basic knowledge of algebra.

#![warn(missing_docs)]

/// Provides the `Expr` type and various methods for working with it
pub mod expr;

mod config;

mod mode;

use crate::expr::Expr;
use anyhow::{Context, Error};
use config::Config;
use std::{
    fmt::Display,
    io::{self, StdinLock, StdoutLock, Write},
    ops::Deref,
};
use termion::cursor::DetectCursorPos;
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
    mode: fn(&mut State<'a>) -> Result<bool, Error>,
    config: Config,
    stdin: Keys<StdinLock<'a>>,
    stdout: RawTerminal<StdoutLock<'a>>,
}

impl<'a> State<'a> {
    fn render(&mut self) -> Result<(), Error> {
        let (_, cy) = self
            .stdout
            .cursor_pos()
            .context("couldn't get cursor pos")?;
        print!("{}{}", clear::CurrentLine, cursor::Goto(0, cy),);

        let mut s = String::new();
        for n in &self.stack {
            s.push_str(&format!("{} ", n));
        }

        s.push_str(&self.input.to_string());

        let (width, ..) = termion::terminal_size().context("couldn't get terminal size")?;
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
        if let Ok(expr) = self.input.parse() {
            self.input.clear();
            self.stack.push(StackItem {
                approx: self.input.contains('.'),
                expr,
            });
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
    where F: Fn(Expr, Expr) -> Expr,
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
        let (cx, cy) = self
            .stdout
            .cursor_pos()
            .context("couldn't get cursor pos")?;
        let (.., height) = termion::terminal_size().context("couldn't get terminal size")?;

        // If the cursor is at the bottom of the screen, make room for one more line.
        if cy == height {
            print!("\n{}", cursor::Goto(cx, cy - 1));
            self.stdout.flush()?;
        }

        loop {
            if (self.mode)(self).context("couldn't tick state")? {
                break;
            };
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
        config: Config::default(),
        stdin,
        stdout,
    };

    state.start().context("couldn't start the event loop")?;

    Ok(())
}
