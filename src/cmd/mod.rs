use crate::{radix::Radix, SoftError, State};

impl<'a> State<'a> {
    /// Process the words after "set" and modify the state.
    pub fn set_cmd<'c, I>(&mut self, words: &mut I) -> Result<(), SoftError>
    where
        I: Iterator<Item = &'c str>,
    {
        match words.next().ok_or(SoftError::GuacCmdMissingArg)? {
            "angle_measure" => {
                let arg = words.next().ok_or(SoftError::GuacCmdExtraArg)?;
                let angle_measure = arg
                    .parse()
                    .map_err(|_| SoftError::BadSetVal(arg.to_owned()))?;
                self.config.angle_measure = angle_measure;
            }
            "radix" => {
                let arg = words.next().ok_or(SoftError::GuacCmdMissingArg)?;
                let radix = arg
                    .parse::<Radix>()
                    .map_err(|_| SoftError::BadSetVal(arg.to_owned()))?;
                self.config.radix = radix;
                for stack_item in &mut self.stack {
                    stack_item.rerender(&self.config);
                }
            }
            "precision" => {
                let arg = words.next().ok_or(SoftError::GuacCmdMissingArg)?;
                let precision = arg
                    .parse::<usize>()
                    .map_err(|_| SoftError::BadSetVal(arg.to_owned()))?;
                self.config.precision = precision;
                for stack_item in &mut self.stack {
                    stack_item.rerender(&self.config);
                }
            }
            other => return Err(SoftError::BadSetPath(other.to_owned())),
        }

        Ok(())
    }

    /// Execute the command currently in `self.input`.
    pub fn exec_cmd(&mut self) -> Result<(), SoftError> {
        let cmd = self.input.clone();
        let mut words = cmd.split_whitespace();
        match words.next().as_deref() {
            Some("set") => self.set_cmd(&mut words)?,
            Some(c) => {
                return Err(SoftError::UnknownGuacCmd(c.to_owned()));
            }
            None => (),
        }

        self.input.clear();

        Ok(())
    }
}
