use crate::{
    expr::{constant::Const, Expr},
    StackItem, State, RADIX,
};
use anyhow::{Context, Result};
use num::{
    traits::{Inv, Pow, Zero},
    BigInt, Signed,
};
use std::{
    io::{self, Write},
    ops::Neg,
};
use termion::{
    clear, color,
    cursor::{self, DetectCursorPos},
    event::Key::*,
    terminal_size,
};

impl<'a> State<'a> {
    /// Write the given mode name on the modeline.
    pub fn write_modeline(&mut self, mut mode: String) -> Result<()> {
        let (width, ..) = terminal_size().context("couldn't get terminal size")?;

        let (cx, cy) = self
            .stdout
            .cursor_pos()
            .context("couldn't get cursor pos")?;

        let line = format!("(q: quit) {}", self.config.angle_measure);

        if !mode.is_empty() {
            mode.insert(0, ' ');
        }

        print!(
            "{}{}{}{}{} {}{}{}{}{}{}{}",
            cursor::Goto(
                1 + width - (1 + self.err.chars().count() + line.len() + mode.len()) as u16,
                cy + 1
            ),
            clear::CurrentLine,
            color::Fg(color::Red),
            self.err,
            color::Fg(color::Reset),
            color::Fg(color::Blue),
            line,
            color::Fg(color::Reset),
            color::Fg(color::Yellow),
            mode,
            color::Fg(color::Reset),
            cursor::Goto(cx, cy),
        );

        self.stdout.flush()?;

        Ok(())
    }

    /// Process a keypress in normal mode.
    pub fn normal(&mut self) -> Result<bool> {
        self.write_modeline(String::new())
            .context("couldn't write modeline")?;
        self.err.clear();

        let key = self.stdin.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "couldn't get next key")
        })??;

        // If the key pressed was a digit in the current radix, update our
        // current input number.
        if let Char(c) = key {
            if c.is_digit(RADIX) || c == '.' || c == 'e' {
                self.input.push(c);
            }
        }

        match key {
            Char('q') | Esc => return Ok(true),
            Char(';') => self.toggle_approx(),
            Char('\n') | Char(' ') => self.push_input(),
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
            Char('/') => {
                if let Ok(n) = self.input.parse::<BigInt>() {
                    if !n.is_zero() {
                        self.apply_binary(|x, y| x / y)
                    } else {
                        self.err = "divide by zero".to_string();
                    }
                } else if let Some(StackItem { expr, .. }) = self.stack.last() {
                    if !expr.is_zero() {
                        self.apply_binary(|x, y| x / y)
                    } else {
                        self.err = "divide by zero".to_string();
                    }
                }
            }
            Char('^') => {
                if let Ok(n) = self.input.parse::<BigInt>() {
                    if n.is_negative() {
                        if let Some(StackItem { expr, .. }) = self.stack.last() {
                            if expr.is_zero() {
                                self.err = "divide by zero".to_string();
                            } else {
                                self.apply_binary(|x, y| x.pow(y))
                            }
                        }
                    } else {
                        self.apply_binary(|x, y| x.pow(y))
                    }
                } else if self.stack[self.stack.len() - 2].expr.is_zero() {
                    self.err = "divide by zero".to_string();
                } else {
                    self.apply_binary(|x, y| x.pow(y))
                }
            }
            Char('l') => self.apply_unary(|x| x.log(Expr::Const(Const::E))),
            Char('L') => self.apply_binary(|x, y| y.log(x)),
            Char('%') => self.apply_binary(|x, y| x % y),
            Char('r') => self.apply_unary(|x| x.sqrt()),
            Char('R') => self.apply_unary(|x| x.pow(2.into())),
            Char('`') => self.apply_unary(|x| x.inv()),
            Char('~') => self.apply_unary(|x| x.neg()),
            Char('|') => self.apply_unary(|x| x.abs()),
            Char('s') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(|x| x.generic_sin(angle_measure))
            }
            Char('c') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(|x| x.generic_cos(angle_measure))
            }
            Char('t') => {
                let angle_measure = self.config.angle_measure;
                if let Ok(n) = self.input.parse::<Expr>() {
                    if n.into_turns(angle_measure) % (1, 2).into() != (1, 4).into() {
                        self.apply_unary(|x| x.generic_tan(angle_measure))
                    } else {
                        self.err = "tangent of π/2".to_string();
                    }
                } else if let Some(n) = self.stack.last() {
                    if n.expr.clone().into_turns(angle_measure) % (1, 2).into() != (1, 4).into() {
                        self.apply_unary(|x| x.generic_tan(angle_measure))
                    } else {
                        self.err = "tangent of π/2".to_string();
                    }
                }
            }
            // Char('c') => self.apply_unary(|x| x.cos()),
            // Char('t') => self.apply_unary(|x| x.tan()),
            // Alt('S') => self.apply_unary(|x| x.asin()),
            // Alt('C') => self.apply_unary(|x| x.acos()),
            // Alt('T') => self.apply_unary(|x| x.atan()),
            Char('x') => self.push_expr(Expr::Var("x".to_string())),
            Char('E') => {
                self.input.push_str("e-");
            }
            Char('k') => self.mode = Self::constant,
            Char('v') => self.mode = Self::variable,
            _ => (),
        };

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Constant mode: push a `Const` to the stack.
    pub fn constant(&mut self) -> Result<bool> {
        self.write_modeline("constant".to_string())
            .context("couldn't write modeline")?;

        let key = self.stdin.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "couldn't get next key")
        })??;

        match key {
            Char('p') => self.push_expr(Expr::Const(Const::Pi)),
            Char('P') => self.push_expr(Expr::Const(Const::Tau)),
            Char('e') => self.push_expr(Expr::Const(Const::E)),
            Char('c') => self.push_expr(Expr::Const(Const::C)),
            Char('G') => self.push_expr(Expr::Const(Const::G)),
            Char('h') => self.push_expr(Expr::Const(Const::H)),
            Char('H') => self.push_expr(Expr::Const(Const::Hbar)),
            Char('k') => self.push_expr(Expr::Const(Const::K)),
            Char('E') => self.push_expr(Expr::Const(Const::Qe)),
            Char('m') => {
                self.mode = Self::mass_constant;
                return Ok(false);
            }
            Char('q') => {
                return Ok(true);
            }
            _ => (),
        };

        self.mode = Self::normal;

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Mass constant mode: sub-mode of constant mode for physical constants which represent the mass of certain particles.
    pub fn mass_constant(&mut self) -> Result<bool> {
        self.write_modeline("mass constant".to_string())
            .context("couldn't write modeline")?;

        let key = self.stdin.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "couldn't get next key")
        })??;

        match key {
            Char('e') => self.push_expr(Expr::Const(Const::Me)),
            Char('p') => self.push_expr(Expr::Const(Const::Mp)),
            Char('q') => {
                return Ok(true);
            }
            _ => (),
        }

        self.mode = Self::normal;

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Variable mode: allows the user to freely type in a custom variable name without triggering single-letter keybinds
    pub fn variable(&mut self) -> Result<bool> {
        self.write_modeline("variable".to_string())
            .context("couldn't write modeline")?;

        let key = self.stdin.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "couldn't get next key")
        })??;

        if let Char(c) = key {
            if c.is_ascii_alphabetic() {
                self.input.push(c);
            }
        }

        match key {
            Char('\n') | Char(' ') => {
                self.push_var();
                self.mode = Self::normal;
            }
            Backspace => {
                self.input.pop();
            }
            Esc => {
                self.input.clear();
                self.mode = Self::normal;
            }
            _ => (),
        }

        self.render().context("couldn't render")?;

        Ok(false)
    }
}
