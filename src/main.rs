use std::io::Write;
use std::io;
use std::error::Error;
use termion::input::TermRead;
use termion::event::Key::*;
use termion::event::Key;
use termion::raw::IntoRawMode;
mod real;

use crate::real::Real;
use termion::{cursor, clear};
use termion::cursor::DetectCursorPos;
// use num_traits::NumOps;

const RADIX: u32 = 10;

struct State<R, W: Write> {
    stack: Vec<Real>,
    input: String,
    stdin: R,
    stdout: W,
}

impl<R: Iterator<Item = Result<Key, std::io::Error>>, W: Write> State<R, W> {
    fn render(&mut self) -> Result<(), std::io::Error> {
        let (_, cy) = self.stdout.cursor_pos()?;
        print!("{}{}", clear::CurrentLine, cursor::Goto(0, cy));
        for n in &self.stack {
            print!("{} ", n);
        }
        print!("{}", self.input);
        Ok(())
    }

    fn apply_binary(&mut self, f: fn(Real, Real) -> Real) -> Result<(), std::io::Error> {
        if let Ok(x) = self.input.parse() {
            if let Some(y) = self.stack.pop() {
                self.stack.push(f(y, x));
                self.input.clear();
                self.render()?;
            }
        } else if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(f(y, x));
                self.render()?;
            }
        }

        Ok(())
    }

    fn apply_unary(&mut self, f: fn(Real) -> Real) -> Result<(), std::io::Error> {
        if let Ok(x) = self.input.parse() {
            self.input.clear();
            self.stack.push(f(x));
            self.render()?;
        } else {
            if let Some(x) = self.stack.pop() {
                self.stack.push(f(x));
                self.render()?;
            }
        }

        Ok(())
    }

    fn dup(&mut self) -> Result<(), std::io::Error> {
        if let Some(l) = self.stack.last() {
            let l = l.clone();
            self.stack.push(l);
            self.render()?;
        }
        Ok(())
    }

    fn swap(&mut self) -> Result<(), std::io::Error> {
        if self.stack.len() >= 2 {
            if let (Some(x), Some(y)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(x);
                self.stack.push(y);
                self.render()?;
            }
        }
        Ok(())
    }

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
                    let i = self.input.parse()?;
                    self.input.clear();
                    self.stack.push(i);
                    print!(" ");
                }
                Char('d') => {
                    self.stack.pop();
                    self.render()?;
                }
                Backspace => {
                    if self.input.pop().is_some() {
                        print!("{} {0}", cursor::Left(1));
                    } else {
                        self.stack.pop();
                        self.render()?;
                    }
                }
                Right => self.swap()?,
                Char('\t') => self.dup()?,
                Char('+') => self.apply_binary(|x, y| x + y)?,
                Char('-') => self.apply_binary(|x, y| x - y)?,
                Char('*') => self.apply_binary(|x, y| x * y)?,
                Char('/') => self.apply_binary(|x, y| x / y)?,
                // Char('^') => self.apply_binary(|x, y| x.powf(y))?,
                // Char('r') => self.apply_unary(|x| x.sqrt())?,
                Alt('r') => self.apply_unary(|x| x * x)?,
                // Char('|') => self.apply_unary(|x| x.abs())?,
                // Char('s') => self.apply_unary(|x| x.sin())?,
                // Char('c') => self.apply_unary(|x| x.cos())?,
                // Char('t') => self.apply_unary(|x| x.tan())?,
                // Alt('S') => self.apply_unary(|x| x.asin())?,
                // Alt('C') => self.apply_unary(|x| x.acos())?,
                // Alt('T') => self.apply_unary(|x| x.atan())?,
                _ => (),
            }

            self.stdout.flush()?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

    state.start()?;

    Ok(())
}
