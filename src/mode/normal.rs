use std::ops::Neg;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use num::{
    traits::{Inv, Pow},
    Signed, Zero,
};

use crate::{
    expr::{constant::Const, Expr},
    SoftError, State, RADIX,
};

use super::Mode;

impl<'a> State<'a> {
    /// Process a keypress in normal mode.
    pub fn normal(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<bool> {
        self.write_modeline().context("couldn't write modeline")?;

        match code {
            KeyCode::Char(c)
                if self.select_idx.is_none() && !self.eex && (c.is_digit(RADIX) || c == '.') =>
            {
                self.input.push(c);
            }
            KeyCode::Char(c)
                if self.select_idx.is_none() && self.eex && (c.is_digit(RADIX) || c == '-') =>
            {
                self.eex_input.push(c);
            }
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char(';') => self.toggle_approx(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.select_idx.is_some() {
                    self.dup();
                } else {
                    let did_push_input = self.push_input();
                    if !did_push_input {
                        self.dup();
                    }
                }
            }
            KeyCode::Char('d') => {
                self.drop();
            }
            KeyCode::Backspace => match &mut self.select_idx {
                None => {
                    if self.eex {
                        if self.eex_input.is_empty() {
                            self.eex = false;
                        } else {
                            self.eex_input.pop();
                        }
                    } else if self.input.is_empty() {
                        self.drop();
                    } else {
                        self.input.pop();
                    }
                }
                Some(i) => {
                    if let Some(j) = i.checked_sub(1) {
                        self.stack.remove(j);
                        *i = i.saturating_sub(1);
                    }
                }
            },
            KeyCode::Right => self.swap(),
            KeyCode::Char('h') => {
                if let Some(i) = &mut self.select_idx {
                    *i = i.saturating_sub(1);
                } else if !self.stack.is_empty() {
                    self.select_idx = Some(self.stack.len() - 1);
                }
            }
            KeyCode::Char('l') => {
                self.select_idx = self.select_idx.map(|x| x + 1);
                if self.select_idx == Some(self.stack.len()) {
                    self.select_idx = None;
                }
            }
            KeyCode::Char('a') => {
                self.select_idx = None;
            }
            KeyCode::Char('+') => self.apply_binary(|x, y| x + y, |_, _| None),
            KeyCode::Char('-') => self.apply_binary(|x, y| x - y, |_, _| None),
            KeyCode::Char('*') => self.apply_binary(|x, y| x * y, |_, _| None),
            KeyCode::Char('/') => self.apply_binary(
                |x, y| x / y,
                |_, y| y.is_zero().then(|| SoftError::DivideByZero),
            ),
            KeyCode::Char('^') => self.apply_binary(Pow::pow, |x, y| {
                (x.is_zero() && y.is_negative()).then(|| SoftError::DivideByZero)
            }),
            KeyCode::Char('g') => self.apply_unary(|x| x.log(Expr::Const(Const::E)), |_| None),
            KeyCode::Char('%') => self.apply_binary(
                |x, y| x % y,
                |_, y| y.is_zero().then(|| SoftError::DivideByZero),
            ),
            KeyCode::Char('r') => {
                self.apply_unary(Expr::sqrt, |x| x.is_negative().then(|| SoftError::Complex));
            }
            KeyCode::Char('`') => {
                self.apply_unary(Inv::inv, |x| x.is_zero().then(|| SoftError::DivideByZero));
            }
            KeyCode::Char('~') => self.apply_unary(Neg::neg, |_| None),
            KeyCode::Char('\\') => self.apply_unary(|x| x.abs(), |_| None),
            KeyCode::Char('s') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(|x| x.generic_sin(angle_measure), |_| None);
            }
            KeyCode::Char('c') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(|x| x.generic_cos(angle_measure), |_| None);
            }
            KeyCode::Char('t') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(
                    |x| x.generic_tan(angle_measure),
                    |x| {
                        (x.clone().into_turns(angle_measure) % Expr::from((1, 2))
                            == Expr::from((1, 4)))
                        .then(|| SoftError::BadTan)
                    },
                );
            }
            KeyCode::Char('x') => self.push_expr(Expr::Var("x".to_string())),
            KeyCode::Char('k') => self.mode = Mode::Constant,
            KeyCode::Char('v') => {
                self.input.clear();
                self.eex_input.clear();
                self.eex = false;
                self.select_idx = None;
                self.mode = Mode::Variable;
            }
            KeyCode::Char('|') => {
                self.push_input();
                if !self.stack.is_empty() {
                    self.err = None;
                    self.input.clear();
                    self.mode = Mode::Pipe;
                }
            }
            KeyCode::Char('e') => self.eex = true,
            // KeyCode::Char('u') => self.undo(),
            // KeyCode::Char('U') => self.redo(),
            KeyCode::Char('<') => {
                if let Some(i) = &mut self.select_idx {
                    if *i != 0 {
                        self.stack.swap(*i, *i - 1);
                        *i -= 1;
                    }
                } else {
                    let did_push_input = self.push_input();
                    if did_push_input {
                        self.swap();
                        self.select_idx = Some(self.stack.len() - 2);
                    }
                }
            }
            KeyCode::Char('>') => {
                if let Some(i) = &mut self.select_idx {
                    if *i < self.stack.len() - 1 {
                        self.stack.swap(*i, *i + 1);
                        *i += 1;
                    }
                }
            }
            KeyCode::Char('G') => self.apply_binary(
                |x, y| y.log(x),
                |_, y| y.is_negative().then(|| SoftError::BadLog),
            ),
            KeyCode::Char('R') => self.apply_unary(|x| x.pow(2.into()), |_| None),
            _ => (),
        }

        Ok(false)
    }
}
