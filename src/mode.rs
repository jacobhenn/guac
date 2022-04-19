use std::io::{self, Write};

use anyhow::{Context, Result};
use num::traits::Pow;
use termion::{
    clear, color,
    cursor::{self, DetectCursorPos},
    event::Key::*,
    terminal_size,
};

use crate::{
    expr::{constant::Const, Expr},
    State, RADIX,
};

impl<'a> State<'a> {
    /// Write the given mode name on the modeline.
    pub fn write_modeline(&mut self, mode: String) -> Result<()> {
        let (width, ..) = terminal_size().context("couldn't get terminal size")?;

        let (cx, cy) = self
            .stdout
            .cursor_pos()
            .context("couldn't get cursor pos")?;

        print!(
            "{}{}{}{}{}{}",
            cursor::Goto(width - mode.len() as u16, cy + 1),
            clear::CurrentLine,
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

        let key = self.stdin.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "couldn't get next key",
        ))??;

        // If the key pressed was a digit in the current radix, update our
        // current input number.
        if let Char(c) = key {
            if c.is_digit(RADIX) || c == '.' {
                self.input.push(c);
            }
        }

        match key {
            Char('q') | Esc | Ctrl('c') => return Ok(true),
            Char('`') => self.toggle_approx(),
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
            Char('/') => self.apply_binary(|x, y| x / y),
            Char('^') => self.apply_binary(|x, y| x.pow(y)),
            // Char('l') => self.apply_unary(|x| x.log(Expr::E)),
            Char('L') => self.apply_binary(|x, y| y.log(x)),
            // Char('r') => state.apply_unary(|x| x.sqrt()),
            // Alt('r') => state.apply_unary(|x| x.pow(Expr::from(2))),
            // Char('n') => state.apply_unary(|x| -x)
            // Char('N') => state.apply_unary(|x| 1/x)
            // Char('|') => state.apply_unary(|x| x.abs()),
            // Char('s') => state.apply_unary(|x| x.sin()),
            // Char('c') => state.apply_unary(|x| x.cos()),
            // Char('t') => state.apply_unary(|x| x.tan()),
            // Alt('S') => state.apply_unary(|x| x.asin()),
            // Alt('C') => state.apply_unary(|x| x.acos()),
            // Alt('T') => state.apply_unary(|x| x.atan()),
            Char('x') => self.push_expr(Expr::Var("x".to_string())),
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

        let key = self.stdin.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "couldn't get next key",
        ))??;

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

        let key = self.stdin.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "couldn't get next key",
        ))??;

        match key {
            Char('e') => self.push_expr(Expr::Const(Const::Me)),
            Char('p') => self.push_expr(Expr::Const(Const::Mp)),
            _ => (),
        }

        self.mode = Self::normal;

        self.render().context("couldn't render")?;

        Ok(false)
    }

    pub fn variable(&mut self) -> Result<bool> {
        self.write_modeline("variable".to_string())
            .context("couldn't write modeline")?;

        let key = self.stdin.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "couldn't get next key",
        ))??;

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
