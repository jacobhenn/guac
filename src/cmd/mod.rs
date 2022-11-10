use crate::{SoftError, State};

impl<'a> State<'a> {
    /// Process the words after "set" and modify the state.
    pub fn set_cmd<I>(&mut self, words: &mut I) -> Result<(), SoftError>
    where
        I: Iterator<Item = String>,
    {
        match words.next() {
            Some(p) if &p == "angle_measure" => {
                if let Some(arg) = words.next() {
                    if let Ok(angle_measure) = arg.parse() {
                        self.config.angle_measure = angle_measure;
                    } else {
                        return Err(SoftError::BadSetVal(arg));
                    }
                } else {
                    return Err(SoftError::GuacCmdMissingArg);
                }
            }
            Some(p) if &p == "radix" => {
                if let Some(arg) = words.next() {
                    if let Ok(radix) = arg.parse() {
                        self.config.radix = radix;
                        for stack_item in &mut self.stack {
                            stack_item.rerender(&self.config);
                        }
                    } else {
                        return Err(SoftError::BadSetVal(arg));
                    }
                }
            }
            Some(p) => {
                return Err(SoftError::BadSetPath(p));
            }
            None => {
                return Err(SoftError::GuacCmdMissingArg);
            }
        }

        Ok(())
    }

    /// Execute the command currently in `self.input`.
    pub fn exec_cmd(&mut self) -> Result<(), SoftError> {
        let cmd = self.input.clone();
        let mut words = cmd.split_whitespace().map(ToString::to_string);
        match words.next().as_deref() {
            Some("set") => self.set_cmd(&mut words)?,
            Some(c) => {
                return Err(SoftError::UnknownGuacCmd(c.to_string()));
            }
            None => (),
        }

        self.input = String::new();

        Ok(())
    }
}
