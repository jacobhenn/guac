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
/// The status returned by a mode funcion when it ticks a keypress.
pub enum Status {
    /// The state has changed, and needs to be rendered again.
    Render,

    /// The user has requested that `guac` exit.
    Exit,

    #[cfg(debug_assertions)]
    /// Debug stuff; this shouldn't compile in release.
    Debug,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
/// A mode that `guac` can be in. All modes interpret keypresses differently.
pub enum Mode {
    /// The default mode, in which the user can manipulate the stack, perform mathematical operations, and type in numbers.
    Normal,

    /// The mode in which the user can push one of several math & physics constants to the stack.
    Constant,

    /// The mode for pushing constants which are the mass of things.
    MassConstant,

    /// The mode in which the user can type in a custom variable name.
    Variable,

    /// The mode in which the user can type in a command into whose stdin the selected (or topmost) expression will be piped.
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
    /// Handle a key event by matching on the current mode.
    pub fn handle_keypress(&mut self, kev: KeyEvent) -> Status {
        match self.mode {
            Mode::Normal => self.normal_mode(kev),
            Mode::Constant => self.constant_mode(kev),
            Mode::MassConstant => self.mass_constant_mode(kev),
            Mode::Variable => self.variable_mode(kev),
            Mode::Pipe => self.pipe_mode(kev),
        }
    }

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
    pub fn constant_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
        match code {
            Char('p') => self.push_expr(Expr::Const(Const::Pi)),
            Char('e') => self.push_expr(Expr::Const(Const::E)),
            Char('c') => self.push_expr(Expr::Const(Const::C)),
            Char('g') => self.push_expr(Expr::Const(Const::Gamma)),
            Char('h') => self.push_expr(Expr::Const(Const::H)),
            Char('k') => self.push_expr(Expr::Const(Const::K)),
            Char('m') => {
                self.mode = Mode::MassConstant;
                return Status::Render;
            }
            Char('H') => self.push_expr(Expr::Const(Const::Hbar)),
            Char('G') => self.push_expr(Expr::Const(Const::G)),
            Char('E') => self.push_expr(Expr::Const(Const::Qe)),
            _ => (),
        }

        self.mode = Mode::Normal;

        Status::Render
    }

    /// Mass constant mode: sub-mode of constant mode for physical constants which represent the mass of certain particles.
    pub fn mass_constant_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
        match code {
            Char('e') => self.push_expr(Expr::Const(Const::Me)),
            Char('p') => self.push_expr(Expr::Const(Const::Mp)),
            _ => (),
        }

        self.mode = Mode::Normal;

        Status::Render
    }

    /// Variable mode: allows the user to freely type in a custom variable name without triggering single-letter keybinds
    pub fn variable_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
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

        Status::Render
    }
}
