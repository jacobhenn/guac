use crate::{State, mode::Status, message::SoftError};

use crossterm::event::{KeyCode, KeyEvent};

impl<'a> State<'a> {
    /// The mode in which the user can enter a `guac` command, such as `set`.
    pub fn cmd_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<Status, SoftError> {
        match code {
            KeyCode::Char(n) => {
                self.input.push(n);
            }
            KeyCode::Backspace => {
                if self.input.is_empty() {
                    self.reset_mode();
                } else {
                    self.input.pop();
                }
            }
            KeyCode::Enter => {
                self.exec_cmd()?;
                self.reset_mode();
            }
            KeyCode::Esc => {
                self.input.clear();
                self.reset_mode();
            }
            _ => (),
        }

        Ok(Status::Render)
    }
}
