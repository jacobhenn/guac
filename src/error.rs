use std::{io, fmt::Display};

/// A representation of an error on the user's end.
#[allow(clippy::module_name_repetitions)]
pub enum SoftError {
    /// Operation would divided by zero.
    DivideByZero,

    /// Operation would produce a complex result, which is not yet supported by `guac`.
    Complex,

    /// Input could not be parsed.
    BadInput,

    /// Eex input (input after the `e` in e-notation) could not be parsed.
    BadEex,

    /// Radix input (input before the `#` in `guac` radix notation) could not be parsed.
    BadRadix,

    /// The argument of `tan` was not in its domain.
    BadTan,

    /// The argument of `log` was not in its domain.
    BadLog,

    /// The command entered in pipe mode could not be run; it returned this IO error.
    BadSysCmd(io::Error),

    /// The command entered in pipe mode failed. The first arg is the name of the command. If it printed to stderr, the second arg contains the first line. If not, it is the `ExitStatus` it returned.
    SysCmdFailed(String, String),

    /// The command entered in pipe mode spawned successfully, but an IO error occurred while attempting to manipulate it.
    SysCmdIoErr(anyhow::Error),

    /// The command entered in command mode was not recognized.
    UnknownGuacCmd(String),

    /// The command entered in command mode was missing an argument.
    GuacCmdMissingArg,

    /// The command entered in command mode had too many arguments.
    GuacCmdExtraArg,

    /// The path provided to the `set` command was bad.
    BadSetPath(String),

    /// The value provided to the `set` command could not be parsed.
    BadSetVal(String),

    /// The input contained a decimal point, but was not in the decimal radix.
    NonDecFloat,

    /// Eex input (input after the `e` in e-notation) was too large to raise an `f64` to the power of.
    BigEex,

    /// This error should never be thrown in a release. It's just used to debug certain things.
    #[cfg(debug_assertions)]
    Debug(String),
}

impl Display for SoftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "E00: divide by zero"),
            Self::Complex => write!(f, "E01: complex not yet supported"),
            Self::BadInput => write!(f, "E02: bad input"),
            Self::BadEex => write!(f, "E03: bad eex input"),
            Self::BadRadix => write!(f, "E04: bad radix"),
            Self::BadTan => write!(f, "E05: tangent of π/2"),
            Self::BadLog => write!(f, "E06: log of n ≤ 0"),
            Self::BadSysCmd(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    write!(f, "E07: unknown command")
                } else {
                    write!(f, "E08: bad command: {e}")
                }
            }
            Self::SysCmdFailed(s, e) => write!(f, "E09: {s}: {e}"),
            Self::SysCmdIoErr(e) => write!(f, "E10: cmd io err: {e}"),
            Self::UnknownGuacCmd(s) => write!(f, "E11: unknown cmd {s}"),
            Self::GuacCmdMissingArg => write!(f, "E12: cmd missing arg"),
            Self::GuacCmdExtraArg => write!(f, "E13: too many cmd args"),
            Self::BadSetPath(p) => write!(f, "E14: no such setting \"{p}\"",),
            Self::BadSetVal(v) => write!(f, "E15: couldnt parse \"{v}\"",),
            Self::NonDecFloat => write!(f, "E16: non-decimal fractional input"),
            Self::BigEex => write!(f, "E17: eex too big"),
            #[cfg(debug_assertions)]
            Self::Debug(s) => write!(f, "DEBUG: {s}"),
        }
    }
}

