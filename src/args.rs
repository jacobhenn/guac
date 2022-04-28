use argh::FromArgs;

#[derive(FromArgs)]
/// `guac` is a minimal but powerful stack-based (RPN) calculator which displays on just a few lines of the terminal.
struct Args {
    #[argh(subcommand)]
    subc: SubCommand
}
