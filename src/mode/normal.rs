use crate::{
    apply_binary, apply_binary_always, apply_unary, apply_unary_always,
    expr::{constant::Const, Expr},
    mode::{Mode, Status},
    error::SoftError, State,
};

use std::ops::Neg;

use crossterm::event::{KeyCode, KeyEvent};

use num::{
    traits::{Inv, Pow},
    One, Signed, Zero,
};

impl<'a> State<'a> {
    /// Process a keypress in normal mode.
    pub fn normal_mode(&mut self, KeyEvent { code, .. }: KeyEvent, escape_digits: bool) -> Status {
        let radix = self.input_radix.unwrap_or(self.config.radix);

        match code {
            KeyCode::Char(c)
                if escape_digits
                    && self.select_idx.is_none()
                    && self.eex_input.is_none()
                    && (radix.contains_digit(&c) || c == '.') =>
            {
                self.input.push(c);
            }
            KeyCode::Char(c)
                if escape_digits
                    && self.select_idx.is_none()
                    && self.eex_input.is_some()
                    && (radix.contains_digit(&c) || c == '-') =>
            {
                self.eex_input.get_or_insert(String::new()).push(c);
            }
            KeyCode::Char('q') => return Status::Exit,
            KeyCode::Esc => {
                if escape_digits {
                    self.mode = Mode::Normal;
                } else {
                    return Status::Exit;
                }
            }
            KeyCode::Char(';') => self.toggle_approx(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.push_input();
            }
            KeyCode::Tab => {
                self.dup();
            }
            KeyCode::Char('d') => {
                self.drop();
            }
            KeyCode::Backspace => match &mut self.select_idx {
                None => {
                    if let Some(eex_input) = &mut self.eex_input {
                        if eex_input.is_empty() {
                            self.eex_input = None;
                        } else {
                            eex_input.pop();
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
            KeyCode::Char('+') => apply_binary_always!(self, |x, y| x + y),
            KeyCode::Char('-') => {
                if let Some(s) = &mut self.eex_input {
                    if s.starts_with('-') {
                        s.remove(0);
                    } else {
                        s.insert(0, '-');
                    }
                } else {
                    apply_binary_always!(self, |x, y| x - y);
                }
            }
            KeyCode::Char('*') => apply_binary_always!(self, |x, y| x * y),
            KeyCode::Char('/') => apply_binary!(self, |x, y| x / y, |_, y| y
                .is_zero()
                .then_some(SoftError::DivideByZero)),
            KeyCode::Char('^') => apply_binary!(self, Pow::pow, |x, y| {
                if x.is_zero() && y.is_negative() {
                    Some(SoftError::DivideByZero)
                } else if x.is_negative() && *y < Expr::one() {
                    Some(SoftError::Complex)
                } else {
                    None
                }
            }),
            KeyCode::Char('g') => apply_unary_always!(self, |x| x.log(Expr::Const(Const::E))),
            KeyCode::Char('%') => apply_binary!(self, |x, y| x % y, |_, y| y
                .is_zero()
                .then_some(SoftError::DivideByZero)),
            KeyCode::Char('r') => {
                apply_unary!(self, Expr::sqrt, |x| {
                    x.is_negative().then_some(SoftError::Complex)
                });
            }
            KeyCode::Char('`') => {
                apply_unary!(self, Inv::inv, |x| x
                    .is_zero()
                    .then_some(SoftError::DivideByZero));
            }
            KeyCode::Char('~') => apply_unary_always!(self, Neg::neg),
            KeyCode::Char('\\') => apply_unary_always!(self, |x| x.abs()),
            KeyCode::Char('s') => {
                let angle_measure = self.config.angle_measure;
                apply_unary_always!(self, |x| x.generic_sin(angle_measure));
            }
            KeyCode::Char('c') => {
                let angle_measure = self.config.angle_measure;
                apply_unary_always!(self, |x| x.generic_cos(angle_measure));
            }
            KeyCode::Char('t') => {
                let angle_measure = self.config.angle_measure;
                apply_unary!(self, |x| x.generic_tan(angle_measure), |x| {
                    (x.clone().into_turns(angle_measure) % Expr::from((1, 2)) == Expr::from((1, 4)))
                        .then_some(SoftError::BadTan)
                });
            }
            KeyCode::Char('S') => {
                let angle_measure = self.config.angle_measure;
                apply_unary!(self, |x| x.asin(angle_measure), |x| {
                    (!x.contains_var() && (x >= &Expr::one() || x <= &Expr::one().neg()))
                        .then_some(SoftError::Complex)
                });
            }
            KeyCode::Char('C') => {
                let angle_measure = self.config.angle_measure;
                apply_unary!(self, |x| x.acos(angle_measure), |x| {
                    (!x.contains_var() && (x <= &Expr::one() || x >= &Expr::one().neg()))
                        .then_some(SoftError::Complex)
                });
            }
            KeyCode::Char('T') => {
                let angle_measure = self.config.angle_measure;
                apply_unary_always!(self, |x| x.atan(angle_measure));
            }
            #[cfg(debug_assertions)]
            KeyCode::Char(']') => {
                unimplemented!("`]` is a debug key which is currently not being used");
            }
            KeyCode::Char('x') => {
                self.push_exact_expr(Expr::Var("x".to_string()), self.config.radix)
            }
            KeyCode::Char('k') => self.mode = Mode::Constant,
            KeyCode::Char('v') => {
                self.input.clear();
                self.eex_input = None;
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
            KeyCode::Char(':') => {
                self.push_input();
                self.err = None;
                self.input.clear();
                self.mode = Mode::Cmd;
            }
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('e') => self.eex_input = Some(String::new()),
            KeyCode::Char('#') => {
                self.radix_input.get_or_insert(String::new());
                self.mode = Mode::Radix;
            }
            KeyCode::Char('u') => return Status::Undo,
            KeyCode::Char('U') => return Status::Redo,
            KeyCode::Char('<') => {
                if let Some(i) = &mut self.select_idx {
                    if *i != 0 {
                        self.stack.swap(*i, *i - 1);
                        *i -= 1;
                    }
                } else
                    if self.push_input().is_some() {
                        self.swap();
                        self.select_idx = Some(self.stack.len() - 2);
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
            KeyCode::Char('G') => apply_binary!(self, |x, y| y.log(x), |_, y| y
                .is_negative()
                .then_some(SoftError::BadLog)),
            KeyCode::Char('R') => apply_unary_always!(self, |x| x.pow(2.into())),
            KeyCode::Char(c)
                if !escape_digits
                    && self.select_idx.is_none()
                    && self.eex_input.is_none()
                    && (radix.contains_digit(&c) || c == '.') =>
            {
                self.input.push(c);
            }
            KeyCode::Char(c)
                if !escape_digits
                    && self.select_idx.is_none()
                    && self.eex_input.is_some()
                    && (radix.contains_digit(&c) || c == '-') =>
            {
                self.eex_input.get_or_insert(String::new()).push(c);
            }
            _ => (),
        }

        Status::Render
    }
}
