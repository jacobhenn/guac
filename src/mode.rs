use num::traits::Pow;
use termion::event::Key::{self, *};

use crate::{expr::Expr, StackItem, State, RADIX};

impl<'a> State<'a> {
    /// Process a keypress in normal mode.
    pub fn normal(&mut self, key: Key) -> bool {
        // If the key pressed was a digit in the current radix, update our
        // current input number.
        if let Char(c) = key {
            if c.is_digit(RADIX) || c == '.' {
                self.input.push(c);
            }
        }

        match key {
            Char('q') | Esc | Ctrl('c') => return true,
            Char('`') => self.toggle_approx(),
            Char('\n') | Char(' ') => self.push(),
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
            Char('x') => self.stack.push(StackItem {
                approx: false,
                expr: Expr::Var("x".to_string()),
            }),
            // Char('e') => self.stack.push(StackItem {
            //     approx: false,
            //     expr: Expr::E,
            // }),
            // Char('P') => self.stack.push(StackItem {
            //     approx: false,
            //     expr: Expr::Tau,
            // }),
            _ => (),
        };

        false
    }
}
