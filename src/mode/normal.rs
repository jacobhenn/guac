use crate::{
    expr::{constant::Const, Expr},
    message::{Message, SoftError},
    mode::{Mode, Status},
    DisplayMode, State,
};

use std::ops::Neg;

use arboard::Clipboard;

use crossterm::event::{KeyCode, KeyEvent};

use num::{
    traits::{Inv, Pow},
    One, Signed, Zero,
};

#[inline]
const fn const_none1<T, R>(_: &T) -> Option<R> {
    None
}

#[inline]
const fn const_none2<T, U, R>(_: &T, _: &U) -> Option<R> {
    None
}

impl<'a> State<'a> {
    /// Process a keypress in normal mode.
    pub fn normal_mode(
        &mut self,
        KeyEvent { code, .. }: KeyEvent,
        escape_digits: bool,
    ) -> Result<Status, SoftError> {
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
            KeyCode::Char('q') => return Ok(Status::Exit),
            KeyCode::Esc => {
                if escape_digits {
                    self.mode = Mode::Normal;
                } else {
                    return Ok(Status::Exit);
                }
            }
            KeyCode::Char(';') => self.toggle_approx(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.push_input()?;
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
            KeyCode::Char('+') => self.apply_binary(&|x, y| x + y, &const_none2)?,
            KeyCode::Char('-') => {
                if let Some(s) = &mut self.eex_input {
                    if s.starts_with('-') {
                        s.remove(0);
                    } else {
                        s.insert(0, '-');
                    }
                } else {
                    self.apply_binary(&|x, y| x - y, &const_none2)?;
                }
            }
            KeyCode::Char('*') => self.apply_binary(&|x, y| x * y, &const_none2)?,
            KeyCode::Char('/') => self.apply_binary(&|x, y| x / y, &|_, y| {
                y.is_zero().then_some(SoftError::DivideByZero)
            })?,
            KeyCode::Char('^') => self.apply_binary(&Pow::pow, &|x, y| {
                if x.is_zero() && y.is_negative() {
                    Some(SoftError::DivideByZero)
                } else if x.is_negative() && *y < Expr::one() {
                    Some(SoftError::Complex)
                } else {
                    None
                }
            })?,
            KeyCode::Char('g') => {
                self.apply_unary(&|x| x.log(Expr::Const(Const::E)), &const_none1)?
            }
            KeyCode::Char('%') => self.apply_binary(&|x, y| x % y, &|_, y| {
                y.is_zero().then_some(SoftError::DivideByZero)
            })?,
            KeyCode::Char('r') => {
                self.apply_unary(&Expr::sqrt, &|x| {
                    x.is_negative().then_some(SoftError::Complex)
                })?;
            }
            KeyCode::Char('`') => {
                self.apply_unary(&Inv::inv, &|x| {
                    x.is_zero().then_some(SoftError::DivideByZero)
                })?;
            }
            KeyCode::Char('~') => self.apply_unary(&Neg::neg, &const_none1)?,
            KeyCode::Char('\\') => self.apply_unary(&|x| x.abs(), &const_none1)?,
            KeyCode::Char('s') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.generic_sin(angle_measure), &const_none1)?;
            }
            KeyCode::Char('c') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.generic_cos(angle_measure), &const_none1)?;
            }
            KeyCode::Char('t') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.generic_tan(angle_measure), &|x| {
                    (x.clone().into_turns(angle_measure) % Expr::from((1, 2)) == Expr::from((1, 4)))
                        .then_some(SoftError::BadTan)
                })?;
            }
            KeyCode::Char('S') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.asin(angle_measure), &|x| {
                    (!x.contains_var() && (x >= &Expr::one() || x <= &Expr::one().neg()))
                        .then_some(SoftError::Complex)
                })?;
            }
            KeyCode::Char('C') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.acos(angle_measure), &|x| {
                    (!x.contains_var() && (x <= &Expr::one() || x >= &Expr::one().neg()))
                        .then_some(SoftError::Complex)
                })?;
            }
            KeyCode::Char('T') => {
                let angle_measure = self.config.angle_measure;
                self.apply_unary(&|x| x.atan(angle_measure), &const_none1)?;
            }
            KeyCode::Char('[') => self.toggle_debug(),
            #[cfg(debug_assertions)]
            KeyCode::Char(']') => {
                self.message = Some(Message::Debug(String::from("debug test :3")));
            }
            KeyCode::Char('x') => {
                self.push_expr(
                    Expr::Var("x".to_string()),
                    self.config.radix,
                    DisplayMode::Exact,
                );
            }
            KeyCode::Char('k') => self.mode = Mode::Constant,
            KeyCode::Char('v') => {
                self.input.clear();
                self.eex_input = None;
                self.select_idx = None;
                self.mode = Mode::Variable;
            }
            KeyCode::Char('|') => {
                self.push_input()?;
                if !self.stack.is_empty() {
                    self.message = None;
                    self.input.clear();
                    self.mode = Mode::Pipe;
                }
            }
            KeyCode::Char(':') => {
                self.push_input()?;
                self.message = None;
                self.input.clear();
                self.mode = Mode::Cmd;
            }
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('e') => self.eex_input = Some(String::new()),
            KeyCode::Char('#') => {
                self.radix_input.get_or_insert(String::new());
                self.mode = Mode::Radix;
            }
            KeyCode::Char('u') => return Ok(Status::Undo),
            KeyCode::Char('U') => return Ok(Status::Redo),
            KeyCode::Char('y') => {
                let Some(e) = self.stack.last() else { return Ok(Status::Render) };
                let mut clipboard = Clipboard::new().map_err(|_| SoftError::Clipboard)?;
                clipboard
                    .set_text(e.display_latex(&self.config))
                    .map_err(|_| SoftError::Clipboard)?;
            }
            KeyCode::Char('<') => {
                if let Some(i) = &mut self.select_idx {
                    if *i != 0 {
                        self.stack.swap(*i, *i - 1);
                        *i -= 1;
                    }
                } else if self.push_input()?.is_some() {
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
            KeyCode::Char('G') => self.apply_binary(&|x, y| y.log(x), &|_, y| {
                y.is_negative().then_some(SoftError::BadLog)
            })?,
            KeyCode::Char('R') => self.apply_unary(&|x| x.pow(2.into()), &const_none1)?,
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

        Ok(Status::Render)
    }
}
