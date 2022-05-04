use crate::{
    expr::{constant::Const, Expr},
    State, RADIX,
};
use anyhow::{Context, Result};
use colored::Colorize;
use crossterm::{
    cursor,
    event::{KeyCode::*, KeyEvent},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use std::fmt::Display;

mod normal;

mod pipe;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Constant,
    MassConstant,
    Variable,
    Pipe,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => Ok(()),
            Self::Constant => write!(f, "enter constant"),
            Self::MassConstant => write!(f, "enter mass constant"),
            Self::Variable => write!(f, "enter variable"),
            Self::Pipe => write!(f, "enter command"),
        }
    }
}

impl<'a> State<'a> {
    /// Write the given mode name on the modeline.
    pub fn write_modeline(&mut self) -> Result<()> {
        let (width, height) = terminal::size().context("couldn't get terminal size")?;

        let (cx, cy) = cursor::position().context("couldn't get cursor pos")?;

        let line = format!(
            "{} {} {} {}",
            self.err
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            "(q: quit)",
            self.config.angle_measure,
            self.mode,
        );

        if line.len() > width as usize {
            return Ok(());
        }

        let colored_line = format!(
            "{} {} {} {}",
            self.err
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
                .red(),
            "(q: quit)",
            self.config.angle_measure,
            self.mode.to_string().yellow().bold(),
        );

        for y in (cy + 1)..height {
            self.stdout
                .queue(cursor::MoveTo(0, y))?
                .queue(terminal::Clear(ClearType::CurrentLine))?;
        }

        self.stdout
            .queue(cursor::MoveTo(width - line.chars().count() as u16, cy + 1))?;

        print!("{}", colored_line);

        self.stdout.execute(cursor::MoveTo(cx, cy))?;

        Ok(())
    }

    /// Constant mode: push a `Const` to the stack.
    pub fn constant(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<bool> {
        match code {
            Char('p') => self.push_expr(Expr::Const(Const::Pi)),
            Char('e') => self.push_expr(Expr::Const(Const::E)),
            Char('c') => self.push_expr(Expr::Const(Const::C)),
            Char('g') => self.push_expr(Expr::Const(Const::Gamma)),
            Char('h') => self.push_expr(Expr::Const(Const::H)),
            Char('k') => self.push_expr(Expr::Const(Const::K)),
            Char('m') => {
                self.mode = Mode::MassConstant;
                return Ok(false);
            }
            Char('q') => {
                return Ok(true);
            }
            Char('P') => self.push_expr(Expr::Const(Const::Tau)),
            Char('H') => self.push_expr(Expr::Const(Const::Hbar)),
            Char('G') => self.push_expr(Expr::Const(Const::G)),
            Char('E') => self.push_expr(Expr::Const(Const::Qe)),
            _ => (),
        }

        self.mode = Mode::Normal;

        Ok(false)
    }

    /// Mass constant mode: sub-mode of constant mode for physical constants which represent the mass of certain particles.
    pub fn mass_constant(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<bool> {
        match code {
            Char('e') => self.push_expr(Expr::Const(Const::Me)),
            Char('p') => self.push_expr(Expr::Const(Const::Mp)),
            Char('q') => {
                return Ok(true);
            }
            _ => (),
        }

        self.mode = Mode::Normal;

        Ok(false)
    }

    /// Variable mode: allows the user to freely type in a custom variable name without triggering single-letter keybinds
    pub fn variable(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<bool> {
        match code {
            Enter | Char(' ') => {
                self.push_var();
                self.mode = Mode::Normal;
            }
            Char(c) if !c.is_digit(RADIX) && !"*+-·/^%()".contains(c) => {
                self.input.push(c);
            }
            Backspace => {
                self.input.pop();
            }
            Esc => {
                self.input.clear();
                self.mode = Mode::Normal;
            }
            _ => (),
        }

        Ok(false)
    }
}
