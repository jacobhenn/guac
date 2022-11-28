use crate::{SoftError, State, radix::Radix};

impl<'a> State<'a> {
    /// Process the words after "set" and modify the state.
    pub fn set_cmd<I>(&mut self, words: &mut I) -> Result<(), SoftError>
    where
        I: Iterator<Item = String>,
    {
        match words.next().ok_or(SoftError::GuacCmdMissingArg)?.as_str() {
            "angle_measure" => {
                let arg = words.next().ok_or(SoftError::GuacCmdExtraArg)?;
                let angle_measure = arg.parse().map_err(|_| SoftError::BadSetVal(arg))?;
                self.config.angle_measure = angle_measure;
            }
            "radix" => {
                let arg = words.next().ok_or(SoftError::GuacCmdMissingArg)?;
                let radix = arg.parse::<Radix>().map_err(|_| SoftError::BadSetVal(arg))?;
                self.config.radix = radix;
                for stack_item in &mut self.stack {
                    stack_item.rerender(&self.config);
                }
            }
            "precision" => {
                let arg = words.next().ok_or(SoftError::GuacCmdMissingArg)?;
                let precision = arg.parse::<usize>().map_err(|_| SoftError::BadSetVal(arg))?;
                self.config.precision = precision;
                for stack_item in &mut self.stack {
                    stack_item.rerender(&self.config);
                }
            }
            other => return Err(SoftError::BadSetPath(other.to_string())),
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
