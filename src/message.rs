use std::{
    borrow::Cow,
    fmt::{self, Display, Write},
    io,
};

use crossterm::style::Stylize;

/// A message that can be displayed to the user on the modeline.
pub enum Message {
    /// The user made an error.
    Error(SoftError),

    // /// The latest operation triggered the complexity heuristics, so it has been forked to another
    // /// thread and can be cancelled at any time.
    // Waiting,
    #[cfg(debug_assertions)]
    /// A debug message for developer use.
    Debug(String),
}

/// A representation of an error on the user's end.
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

    /// Eex input (input after the `e` in e-notation) was too large to raise an `f64` to the power of.
    BigEex,

    /// An error occurred when interacting with the clipboard.
    Clipboard,

    /// Some parts of stdin could not be parsed into numbers.
    StdinParse(Vec<usize>),
}

impl SoftError {
    /// The unique code of this error. If 1.0 ever releases, error codes will be fixed and
    /// forward-compatible. Until then, they can change all they want.
    pub fn code(&self) -> usize {
        match self {
            SoftError::DivideByZero => 0,
            SoftError::Complex => 1,
            SoftError::BadInput => 2,
            SoftError::BadEex => 3,
            SoftError::BadRadix => 4,
            SoftError::BadTan => 5,
            SoftError::BadLog => 6,
            SoftError::BadSysCmd(_) => 7,
            SoftError::SysCmdFailed(_, _) => 8,
            SoftError::SysCmdIoErr(_) => 9,
            SoftError::UnknownGuacCmd(_) => 10,
            SoftError::GuacCmdMissingArg => 11,
            SoftError::GuacCmdExtraArg => 12,
            SoftError::BadSetPath(_) => 13,
            SoftError::BadSetVal(_) => 14,
            SoftError::BigEex => 15,
            SoftError::Clipboard => 16,
            SoftError::StdinParse(_) => 17,
        }
    }
}

fn strclamp(s: &str, len: usize) -> Cow<str> {
    if s.len() <= len {
        Cow::Borrowed(s)
    } else {
        let i = s
            .char_indices()
            .take(len)
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        Cow::Owned(format!("{}…", &s[..=i]))
    }
}

/// Display the list of values separated by commas, but cut off the list when displaying a new
// element would make the resulting string exceed `len`.
fn listclamp<T>(values: &[T], len: usize) -> Result<String, fmt::Error>
where
    T: Display,
{
    let mut s = String::new();
    let mut prev_len = 0;
    let mut values = values.into_iter().peekable();
    while let Some(value) = values.next() {
        write!(&mut s, "{value}")?;
        if s.len() > len {
            s.truncate(prev_len);
            s.push_str("…");
            return Ok(s);
        }

        if values.peek().is_some() {
            s.push_str(", ");
        }

        prev_len = s.len();
    }
    Ok(s)
}

fn plural(len: usize) -> &'static str {
    if len == 1 {
        ""
    } else {
        "s"
    }
}

impl Display for SoftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:0>2}: ", self.code())?;
        match self {
            Self::DivideByZero => f.write_str("divide by zero"),
            Self::Complex => f.write_str("complex not yet supported"),
            Self::BadInput => f.write_str("bad input"),
            Self::BadEex => f.write_str("bad eex input"),
            Self::BadRadix => f.write_str("bad radix"),
            Self::BadTan => f.write_str("tangent of π/2"),
            Self::BadLog => f.write_str("log of n ≤ 0"),
            Self::BadSysCmd(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    f.write_str("unknown command")
                } else {
                    write!(f, "bad command: {e}")
                }
            }
            Self::SysCmdFailed(s, e) => write!(f, "{}: {}", strclamp(s, 18), strclamp(e, 24)),
            Self::SysCmdIoErr(e) => write!(f, "cmd io err: {e}"),
            Self::UnknownGuacCmd(s) => write!(f, "unknown cmd {s}"),
            Self::GuacCmdMissingArg => f.write_str("cmd missing arg"),
            Self::GuacCmdExtraArg => f.write_str("too many cmd args"),
            Self::BadSetPath(p) => write!(f, r#"no such setting "{}""#, strclamp(p, 18)),
            Self::BadSetVal(v) => write!(f, r#"couldnt parse "{}""#, strclamp(v, 18)),
            Self::BigEex => f.write_str("eex too big"),
            Self::Clipboard => f.write_str("clipboard error"),
            Self::StdinParse(line) => write!(
                f,
                "couldnt parse stdin line{} {}",
                plural(line.len()),
                listclamp(&line, 18)?,
            ),
        }
    }
}

// const WAITING_MSG: &str = "waiting... (esc: cancel)";

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Error(e) => e.fmt(f),
            // Message::Waiting => WAITING_MSG.yellow().fmt(f),
            #[cfg(debug_assertions)]
            Message::Debug(m) => f.write_str(m),
        }
    }
}

impl Message {
    /// Render the message in color.
    pub fn to_colored_string(&self) -> String {
        match self {
            Message::Error(e) => e.to_string().red().to_string(),
            // Message::Waiting => "waiting... (esc: cancel)".yellow().to_string(),
            Message::Debug(m) => m.as_str().blue().to_string(),
        }
    }
}
