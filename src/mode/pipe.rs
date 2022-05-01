use crate::{SoftError, State};
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use std::{
    io::{BufRead, BufReader, Write},
    mem,
    process::{self, Stdio},
};

use super::Mode;

impl<'a> State<'a> {
    /// Execute the command entered in pipe mode.
    ///
    /// # Panics
    ///
    /// This function will panic and/or do weird things if not called in pipe mode.
    pub fn execute_pipe(&mut self) -> Result<()> {
        let mut words = self.input.split_whitespace();
        if let Some(word) = words.next() {
            let mut cmd = process::Command::new(word);
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::piped());

            for word in words {
                cmd.arg(word);
            }

            match cmd.spawn() {
                Ok(mut child) => {
                    let mut stdin = child.stdin.take().context("failed to open child stdin")?;
                    let stderr = child.stderr.take().context("failed to open child stderr")?;
                    let expr = if let Some(i) = self.select_idx {
                        self.stack[i].expr.clone()
                    } else {
                        self.stack.last().unwrap().expr.clone()
                    };

                    stdin
                        .write_all(expr.to_string().as_bytes())
                        .context("failed to write to child stdin")?;
                    mem::drop(stdin);

                    let status = child.wait().context("failed to get child's exit status")?;
                    if !status.success() {
                        let stderr = BufReader::new(stderr);
                        self.err = Some(SoftError::CmdFailed(
                            word.to_string(),
                            stderr
                                .lines()
                                .next()
                                .unwrap_or_else(|| Ok(status.to_string()))
                                .context("failed to read child stderr")?,
                        ));
                    }
                }
                Err(e) => {
                    self.err = Some(SoftError::BadCmd(e));
                }
            }
        }

        Ok(())
    }

    /// Process a keypress in pipe mode.
    pub fn pipe_mode(&mut self, KeyEvent { code, .. }: KeyEvent) -> Result<bool> {
        match code {
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Enter => {
                self.execute_pipe().context("couldn't execute command")?;

                if self.err.is_none() {
                    self.input.clear();
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Backspace => {
                if self.input.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.input.pop();
                }
            }
            KeyCode::Esc => {
                self.input.clear();
                self.mode = Mode::Normal;
            }
            _ => (),
        }

        Ok(false)
    }
}
