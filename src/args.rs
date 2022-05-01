use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// A minimal but powerful interactive [stack-based](https://en.wikipedia.org/wiki/Reverse_Polish_notation) calculator which displays on just a few lines of the terminal.
pub struct Args {
    #[argh(subcommand)]
    pub subc: Option<SubCommand>
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommand {
    Keys(Keys),
    Anyway(Anyway),
}

#[derive(FromArgs, PartialEq, Debug)]
/// print a list of keybindings and their actions
#[argh(subcommand, name = "keys")]
pub struct Keys {}

#[derive(FromArgs, PartialEq, Debug)]
/// disregard inconveniences
#[argh(subcommand, name = "anyway")]
pub struct Anyway {}
