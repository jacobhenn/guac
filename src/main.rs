use anyhow::{Context, Error};
use num::BigInt;
use std::io;
use std::io::Write;
use termion::event::Key;
use termion::event::Key::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
mod expr;

use crate::expr::Expr;
use termion::cursor::DetectCursorPos;
use termion::{clear, cursor};
// use num_traits::NumOps;

const RADIX: u32 = 10;

struct State<R, W: Write> {
    stack: Vec<Expr>,
    input: String,
    stdin: R,
    stdout: W,
}

impl<R: Iterator<Item = Result<Key, std::io::Error>>, W: Write> State<R, W> {
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

    fn apply_binary(&mut self, f: fn(Expr, Expr) -> Expr) -> () {
        if let Ok(x) = self.input.parse() {
            if self.stack.len() >= 1 {
                self.stack.push(x);
                self.input.clear();
            }
        }

        if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(f(y, x));
            }
        }
    }

    fn apply_unary(&mut self, f: fn(Expr) -> Expr) -> () {
        if let Ok(x) = self.input.parse() {
            self.stack.push(x);
            self.input.clear();
        }

        if self.stack.len() >= 1 {
            if let Some(x) = self.stack.pop() {
                self.stack.push(f(x));
            }
        }
    }

    fn dup(&mut self) -> () {
        if let Some(l) = self.stack.pop() {
            self.stack.push(l.clone());
            self.stack.push(l);
        }
    }

    fn swap(&mut self) -> () {
        if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(x);
                self.stack.push(y);
            }
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        loop {
            let key = self.stdin.next().unwrap()?;

            // If the key pressed was a digit in the current radix, update our
            // current input number.
            if let Char(c) = key {
                if c.is_digit(RADIX) || c == '.' {
                    self.input.push(c);
                    print!("{}", c);
                }
            }

            match key {
                Char('q') | Esc | Ctrl('c') => break,
                Char('\n') | Char(' ') => {
                    if let Ok(i) = self.input.parse() {
                        self.input.clear();
                        self.stack.push(i);
                    } else {
                        continue;
                    }
                }
                Char('d') => {
                    self.stack.pop();
                }
                Backspace => {
                    if self.input.is_empty() {
                        self.stack.pop();
                    } else {
                        self.input.pop();
                    }
                }
                Right => self.swap(),
                Char('\t') => self.dup(),
                Char('+') => self.apply_binary(|x, y| x + y),
                Char('-') => self.apply_binary(|x, y| x - y),
                Char('*') => self.apply_binary(|x, y| x * y),
                Char('/') => self.apply_binary(|x, y| x / y),
                // Char('^') => self.apply_binary(|x, y| x.powf(y)),
                // Char('r') => self.apply_unary(|x| x.sqrt()),
                Alt('r') => self.apply_unary(|x| x.pow(Expr::Int(BigInt::from(2)))),
                // Char('|') => self.apply_unary(|x| x.abs()),
                // Char('s') => self.apply_unary(|x| x.sin()),
                // Char('c') => self.apply_unary(|x| x.cos()),
                // Char('t') => self.apply_unary(|x| x.tan()),
                // Alt('S') => self.apply_unary(|x| x.asin()),
                // Alt('C') => self.apply_unary(|x| x.acos()),
                // Alt('T') => self.apply_unary(|x| x.atan()),
                _ => (),
            }

            self.render()?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let stdout = io::stdout();
    // if !stdout.is_tty() {
    //     eprintln!("stdout is not a tty!");
    //     return Ok(());
    // }
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode()?;

    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut state = State {
        stack: Vec::new(),
        input: String::new(),
        stdin: stdin.keys(),
        stdout: stdout.into_raw_mode()?,
    };

    state.start().context("couldn't start the event loop")?;

    Ok(())
}
