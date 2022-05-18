use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// A minimal but powerful interactive stack-based calculator which displays on just a few lines of the terminal.
pub struct Args {
    #[argh(switch)]
    /// don't check width, istty, etc
    pub force: bool,

    #[argh(subcommand)]
    pub subc: Option<SubCommand>
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommand {
    Keys(Keys),
}

#[derive(FromArgs, PartialEq, Debug)]
/// print a list of keybindings and their actions
#[argh(subcommand, name = "keys")]
pub struct Keys {}
