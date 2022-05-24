use crate::{SoftError, State};

impl<'a> State<'a> {
    /// Process the words after "set" and modify the state.
    pub fn set_cmd<I>(&mut self, words: &mut I)
    where
        I: Iterator<Item = String>,
    {
        match words.next() {
            Some(p) if &p == "angle_measure" => {
                if let Some(arg) = words.next() {
                    if let Ok(angle_measure) = arg.parse() {
                        self.config.angle_measure = angle_measure;
                    } else {
                        self.err = Some(SoftError::BadSetVal(arg));
                    }
                } else {
                    self.err = Some(SoftError::GuacCmdMissingArg);
                }
            }
            Some(p) => {
                self.err = Some(SoftError::BadSetPath(p));
            }
            None => {
                self.err = Some(SoftError::GuacCmdMissingArg);
            }
        }
    }

    /// Execute the command currently in `self.input`.
    pub fn exec_cmd(&mut self) {
        let cmd = self.input.clone();
        let mut words = cmd.split_whitespace().map(ToString::to_string);
        match words.next().as_deref() {
            Some("set") => self.set_cmd(&mut words),
            Some(c) => {
                self.err = Some(SoftError::UnknownGuacCmd(c.to_string()));
            }
            None => (),
        }

        if self.err.is_none() {
            self.input.clear();
        }
    }
}
