use crate::{
    expr::{constant::Const, Expr},
    StackItem, State, RADIX,
};
use anyhow::{Context, Result};
use colored::Colorize;
use crossterm::{
    cursor,
    event::{self, Event::*, KeyCode::*, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use num::{
    traits::{Inv, Pow, Zero},
    BigInt, Signed,
};
use std::ops::Neg;

pub enum Mode {
    Normal(),
    Constant(),
    MassConstant(),
    Variable(),
}

impl<'a> State<'a> {
    /// Write the given mode name on the modeline.
    pub fn write_modeline(&mut self, mode: &str) -> Result<()> {
        let (width, height) = terminal::size().context("couldn't get terminal size")?;

        let (cx, cy) = cursor::position().context("couldn't get cursor pos")?;

        let line = format!(
            "{} {} {} {}",
            self.err, "(q: quit)", self.config.angle_measure, mode,
        );

        let colored_line = format!(
            "{} {} {} {}",
            self.err.red(),
            "(q: quit)".blue(),
            self.config.angle_measure.to_string().blue(),
            mode.yellow(),
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

    /// Process a keypress in normal mode.
    pub fn normal(&mut self) -> Result<bool> {
        self.write_modeline("").context("couldn't write modeline")?;
        self.err.clear();

        if let Key(KeyEvent { code, modifiers }) = event::read()? {
            if modifiers.is_empty() {
                match code {
                    Char(c)
                        if self.select_idx.is_none()
                            && self.eex
                            && (c.is_digit(RADIX) || c == '-') =>
                    {
                        self.eex_input.push(c)
                    }
                    Char(c)
                        if self.select_idx.is_none()
                            && !self.eex
                            && (c.is_digit(RADIX) || c == '.') =>
                    {
                        self.input.push(c)
                    }
                    Char('q') | Esc => return Ok(true),
                    Char(';') => self.toggle_approx(),
                    Enter | Char(' ') => {
                        self.push_input();
                    }
                    Char('d') => {
                        self.drop();
                    }
                    Backspace => match &mut self.select_idx {
                        None => {
                            if self.eex {
                                if self.eex_input.is_empty() {
                                    self.eex = false;
                                } else {
                                    self.eex_input.pop();
                                }
                            } else {
                                if self.input.is_empty() {
                                    self.drop();
                                } else {
                                    self.input.pop();
                                }
                            }
                        }
                        Some(i) => {
                            if let Some(j) = i.checked_sub(1) {
                                self.stack.remove(j);
                                *i = i.saturating_sub(1);
                            }
                        }
                    },
                    Right => self.swap(),
                    Char('h') => {
                        if let Some(i) = &mut self.select_idx {
                            *i = i.saturating_sub(1);
                        } else if !self.stack.is_empty() {
                            self.select_idx = Some(self.stack.len() - 1);
                        }
                    }
                    Char('l') => {
                        self.select_idx = self.select_idx.map(|x| x + 1);
                        if self.select_idx == Some(self.stack.len()) {
                            self.select_idx = None;
                        }
                    }
                    Char('a') => {
                        self.select_idx = None;
                    }
                    Char('\t') => self.dup(),
                    Char('+') => self.apply_binary(|x, y| x + y),
                    Char('-') => self.apply_binary(|x, y| x - y),
                    Char('*') => self.apply_binary(|x, y| x * y),
                    Char('/') => {
                        if let Ok(n) = self.input.parse::<BigInt>() {
                            if n.is_zero() {
                                self.err = "divide by zero".to_string();
                            } else {
                                self.apply_binary(|x, y| x / y);
                            }
                        } else if let Some(StackItem { expr, .. }) = self.stack.last() {
                            if expr.is_zero() {
                                self.err = "divide by zero".to_string();
                            } else {
                                self.apply_binary(|x, y| x / y);
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
                                        self.apply_binary(Pow::pow);
                                    }
                                }
                            } else {
                                self.apply_binary(Pow::pow);
                            }
                        } else if self.stack[self.stack.len() - 2].expr.is_zero() {
                            self.err = "divide by zero".to_string();
                        } else {
                            self.apply_binary(Pow::pow);
                        }
                    }
                    Char('g') => self.apply_unary(|x| x.log(Expr::Const(Const::E))),
                    Char('%') => self.apply_binary(|x, y| x % y),
                    Char('r') => self.apply_unary(Expr::sqrt),
                    Char('`') => self.apply_unary(Inv::inv),
                    Char('~') => self.apply_unary(Neg::neg),
                    Char('|') => self.apply_unary(|x| x.abs()),
                    Char('s') => {
                        let angle_measure = self.config.angle_measure;
                        self.apply_unary(|x| x.generic_sin(angle_measure));
                    }
                    Char('c') => {
                        let angle_measure = self.config.angle_measure;
                        self.apply_unary(|x| x.generic_cos(angle_measure));
                    }
                    Char('t') => {
                        let angle_measure = self.config.angle_measure;
                        if let Ok(n) = self.input.parse::<Expr>() {
                            if n.into_turns(angle_measure) % (1, 2).into() == (1, 4).into() {
                                self.err = "tangent of π/2".to_string();
                            } else {
                                self.apply_unary(|x| x.generic_tan(angle_measure));
                            }
                        } else if let Some(n) = self.stack.last() {
                            if n.expr.clone().into_turns(angle_measure) % (1, 2).into()
                                == (1, 4).into()
                            {
                                self.err = "tangent of π/2".to_string();
                            } else {
                                self.apply_unary(|x| x.generic_tan(angle_measure));
                            }
                        }
                    }
                    Char('x') => self.push_expr(Expr::Var("x".to_string())),
                    Char('k') => self.mode = Mode::Constant(),
                    Char('v') => {
                        self.input.clear();
                        self.eex_input.clear();
                        self.eex = false;
                        self.mode = Mode::Variable();
                    }
                    Char('e') => self.eex = true,
                    Char('<') => {
                        if let Some(i) = &mut self.select_idx {
                            if *i != 0 {
                                self.stack.swap(*i, *i - 1);
                                *i -= 1;
                            }
                        } else {
                            if self.push_input() {
                                self.swap();
                                self.select_idx = Some(self.stack.len() - 2);
                            }
                        }
                    }
                    Char('>') => {
                        if let Some(i) = &mut self.select_idx {
                            if *i < self.stack.len() - 1 {
                                self.stack.swap(*i, *i + 1);
                                *i += 1;
                            }
                        }
                    }
                    _ => (),
                }
            } else if modifiers == KeyModifiers::SHIFT {
                match code {
                    Char('G') => self.apply_binary(|x, y| y.log(x)),
                    Char('R') => self.apply_unary(|x| x.pow(2.into())),
                    _ => (),
                }
            }
        }

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Constant mode: push a `Const` to the stack.
    pub fn constant(&mut self) -> Result<bool> {
        self.write_modeline("constant")
            .context("couldn't write modeline")?;

        if let Key(KeyEvent { code, modifiers }) = event::read()? {
            if modifiers.is_empty() {
                match code {
                    Char('p') => self.push_expr(Expr::Const(Const::Pi)),
                    Char('e') => self.push_expr(Expr::Const(Const::E)),
                    Char('c') => self.push_expr(Expr::Const(Const::C)),
                    Char('h') => self.push_expr(Expr::Const(Const::H)),
                    Char('k') => self.push_expr(Expr::Const(Const::K)),
                    Char('m') => {
                        self.mode = Mode::MassConstant();
                        return Ok(false);
                    }
                    Char('q') => {
                        return Ok(true);
                    }
                    _ => (),
                }
            } else if modifiers == KeyModifiers::SHIFT {
                match code {
                    Char('P') => self.push_expr(Expr::Const(Const::Tau)),
                    Char('H') => self.push_expr(Expr::Const(Const::Hbar)),
                    Char('G') => self.push_expr(Expr::Const(Const::G)),
                    Char('E') => self.push_expr(Expr::Const(Const::Qe)),
                    _ => (),
                }
            }
        };

        self.mode = Mode::Normal();

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Mass constant mode: sub-mode of constant mode for physical constants which represent the mass of certain particles.
    pub fn mass_constant(&mut self) -> Result<bool> {
        self.write_modeline("mass constant")
            .context("couldn't write modeline")?;

        if let Key(KeyEvent { code, modifiers }) = event::read()? {
            if modifiers.is_empty() {
                match code {
                    Char('e') => self.push_expr(Expr::Const(Const::Me)),
                    Char('p') => self.push_expr(Expr::Const(Const::Mp)),
                    Char('q') => {
                        return Ok(true);
                    }
                    _ => (),
                }
            }
        }

        self.mode = Mode::Normal();

        self.render().context("couldn't render")?;

        Ok(false)
    }

    /// Variable mode: allows the user to freely type in a custom variable name without triggering single-letter keybinds
    pub fn variable(&mut self) -> Result<bool> {
        self.write_modeline("variable")
            .context("couldn't write modeline")?;

        if let Key(KeyEvent { code, modifiers }) = event::read()? {
            if modifiers.is_empty() {
                match code {
                    Char(c) if !c.is_digit(RADIX) && !"+-·/^%()".contains(c) => {
                        self.input.push(c);
                    }
                    Enter | Char(' ') => {
                        self.push_var();
                        self.mode = Mode::Normal();
                    }
                    Backspace => {
                        self.input.pop();
                    }
                    Esc => {
                        self.input.clear();
                        self.mode = Mode::Normal();
                    }
                    _ => (),
                }
            }
        }

        self.render().context("couldn't render")?;

        Ok(false)
    }
}
