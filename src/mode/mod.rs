use crate::{
    expr::{constant::Const, Expr},
    radix::{self, Radix},
    error::SoftError, State,
};

use std::fmt::Display;

use anyhow::{Context, Result};

use colored::Colorize;

use crossterm::{
    cursor,
    event::{KeyCode::*, KeyEvent},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

mod normal;

mod pipe;

mod cmd;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
/// A message from the current mode to the event loop that tells it what to do.
pub enum Status {
    /// The state has been changed, and needs to be rendered again.
    Render,

    /// The user has requested that `guac` exit.
    Exit,

    /// The user pressed the `undo` key.
    Undo,

    /// The user pressed the `redo` key.
    Redo,

    #[cfg(debug_assertions)]
    /// Debug stuff; this shouldn't compile in release.
    Debug,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
/// A mode that `guac` can be in. All modes interpret keypresses differently.
pub enum Mode {
    /// The default mode, in which the user can manipulate the stack, perform mathematical operations, and type in numbers.
    ///
    /// Tries to interpret keys as binds before digits.
    Normal,

    /// Tries to interpret keys as digits before binds.
    Insert,

    /// The mode in which the user can push one of several math & physics constants to the stack.
    Constant,

    /// The mode for pushing constants which are the mass of things.
    MassConstant,

    /// The mode in which the user can type in a custom variable name.
    Variable,

    /// The mode in which the user can type in a command into whose stdin the selected (or topmost) expression will be piped.
    Pipe,

    /// The mode in which the user can type in a radix in which to input a number.
    Radix,

    /// The mode in which the user can type in a `guac` command, such as `set`.
    Cmd,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => Ok(()),
            Self::Insert => write!(f, "insert"),
            Self::Constant => write!(f, "enter constant"),
            Self::MassConstant => write!(f, "enter mass constant"),
            Self::Variable => write!(f, "enter variable"),
            Self::Radix => write!(f, "enter radix"),
            Self::Pipe | Self::Cmd => write!(f, "enter command"),
        }
    }
}

impl<'a> State<'a> {
    /// If the current radix is greater than decimal, set the mode to input. Else, set the mode to normal.
    pub fn reset_mode(&mut self) {
        if self.input_radix.unwrap_or(self.config.radix) > Radix::DECIMAL {
            self.mode = Mode::Insert;
        } else {
            self.mode = Mode::Normal;
        }
    }

    /// Handle a key event by matching on the current mode.
    pub fn handle_keypress(&mut self, kev: KeyEvent) -> Status {
        match self.mode {
            Mode::Normal => self.normal_mode(kev, false),
            Mode::Insert => self.normal_mode(kev, true),
            Mode::Constant => self.constant_mode(kev),
            Mode::MassConstant => self.mass_constant_mode(kev),
            Mode::Variable => self.variable_mode(kev),
            Mode::Pipe => self.pipe_mode(kev),
            Mode::Radix => self.radix_mode(kev),
            Mode::Cmd => self.cmd_mode(kev),
        }
    }

    /// Write the given mode name on the modeline.
    pub fn write_modeline(&mut self) -> Result<()> {
        let (width, height) = terminal::size().context("couldn't get terminal size")?;

        let (cx, cy) = cursor::position().context("couldn't get cursor pos")?;

        let line = format!(
            "{} {} {} {} {}",
            self.err
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            "(q: quit)",
            self.config.angle_measure,
            self.config.radix,
            self.mode,
        );

        if line.len() > width as usize {
            return Ok(());
        }

        let colored_line = format!(
            "{} {} {} {} {}",
            self.err
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
                .red(),
            "(q: quit)",
            self.config.angle_measure,
            self.config.radix,
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

    /// Push an exact expression containing the constant `c` to the stack.
    pub fn push_const(&mut self, c: Const) {
        self.push_exact_expr(Expr::Const(c), self.config.radix);
    }

    /// Constant mode: push a `Const` to the stack.
    pub fn constant_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
        match code {
            Char('p') => self.push_const(Const::Pi),
            Char('e') => self.push_const(Const::E),
            Char('c') => self.push_const(Const::C),
            Char('g') => self.push_const(Const::Gamma),
            Char('h') => self.push_const(Const::H),
            Char('k') => self.push_const(Const::K),
            Char('m') => {
                self.mode = Mode::MassConstant;
                return Status::Render;
            }
            Char('H') => self.push_const(Const::Hbar),
            Char('G') => self.push_const(Const::G),
            Char('E') => self.push_const(Const::Qe),
            _ => (),
        }

        self.mode = Mode::Normal;

        Status::Render
    }

    /// Mass constant mode: sub-mode of constant mode for physical constants which represent the mass of certain particles.
    pub fn mass_constant_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
        match code {
            Char('e') => self.push_const(Const::Me),
            Char('p') => self.push_const(Const::Mp),
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
            Char(c) if !self.config.radix.contains_digit(&c) && !"#*+-·/^%()".contains(c) => {
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

    /// Radix mode: allows the user to type in a radix in which to input a number
    pub fn radix_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Status {
        match code {
            Enter | Char(' ' | '#') => {
                if let Ok(radix) = self
                    .radix_input
                    .clone()
                    .unwrap_or_default()
                    .parse::<Radix>()
                {
                    self.input_radix = Some(radix);
                    self.reset_mode();
                } else if self
                    .radix_input
                    .as_ref()
                    .map(String::is_empty)
                    .unwrap_or_default()
                {
                    self.radix_input = None;
                    self.input_radix = None;
                    self.mode = Mode::Normal;
                } else {
                    self.err = Some(SoftError::BadRadix);
                }
            }
            Char(c) if radix::DIGITS.contains(&c) => {
                self.radix_input.get_or_insert(String::new()).push(c);
            }
            Backspace => {
                if let Some(radix_input) = &mut self.radix_input {
                    if radix_input.is_empty() {
                        self.stack.pop();
                    } else {
                        radix_input.pop();
                    }
                }
            }
            Esc => {
                self.radix_input = None;
                self.input_radix = None;
                self.reset_mode();
            }
            _ => (),
        }

        Status::Render
    }
}

/// An unpleasent helper macro for [`State::apply_binary`] that will hopefully go away soon
#[macro_export]
macro_rules! apply_binary {
    ( $state:expr, $f:expr, $domain:expr ) => {
        $state.apply_binary($f, $f, $domain, $domain)
    }
}

/// An unpleasent helper macro for [`State::apply_binary`] that will hopefully go away soon
#[macro_export]
macro_rules! apply_binary_always {
    ( $state:expr, $f:expr ) => {
        $state.apply_binary($f, $f, |_, _| None, |_, _| None)
    }
}

/// An unpleasent helper macro for [`State::apply_binary`] that will hopefully go away soon
#[macro_export]
macro_rules! apply_unary {
    ( $state:expr, $f:expr, $domain:expr ) => {
        $state.apply_unary($f, $f, $domain, $domain)
    }
}

/// An unpleasent helper macro for [`State::apply_binary`] that will hopefully go away soon
#[macro_export]
macro_rules! apply_unary_always {
    ( $state:expr, $f:expr ) => {
        $state.apply_unary($f, $f, |_| None, |_| None)
    }
}
