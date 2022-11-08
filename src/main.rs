//! Generally Underappreciated Algebraic Calculator
//!
//! `guac` is a minimal stack-based (RPN) calculator with a basic knowledge of algebra.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

use crate::{
    args::{Args, SubCommand},
    config::Config,
    expr::Expr,
    mode::{Mode, Status},
    radix::Radix,
};

use std::{
    fmt::Display,
    io::{self, stdin, stdout, BufRead, BufReader, ErrorKind, StdoutLock, Write},
    mem,
    process::exit,
};

use anyhow::{bail, Context, Error};

use colored::Colorize;

use crossterm::{
    cursor,
    event::{self, Event},
    terminal::{self, ClearType},
    tty::IsTty,
    ExecutableCommand, QueueableCommand,
};

use num::{traits::Pow, BigRational};

/// Provides the `Expr` type and various methods for working with it
pub mod expr;

/// Structures into which configuration is parsed.
pub mod config;

/// Types and functions for keeping track of and executing modes.
pub mod mode;

/// Types and functions for executing in-guac commands.
pub mod cmd;

/// Types and functions for parsing and displaying radices.
pub mod radix;

mod args;

#[cfg(test)]
mod tests;

// macro_rules! trait_alias {
//     ( $name:ident: $traits:tt; ) => {
//         trait $name: $traits {}

//         impl<T> $name for T where T: $($tr +)+ {}
//     }
// }

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
                if e.kind() == ErrorKind::NotFound {
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

/// Either an exact expression (`Expr<BigRational>`) or an approximate one (`Expr<f64>`).
#[derive(Clone, PartialEq, Debug)]
pub enum StackExpr {
    /// An exact expression.
    Exact(Expr<BigRational>),

    /// An approximate expression.
    Approx(Expr<f64>),
}

/// The component of [`StackItem`] that is dependent on the exactness of its backing expression.
#[derive(Clone, PartialEq, Debug)]
pub enum StackVal {
    /// A stack value that is backed by an exact expression, and can therefore be displayed as
    /// either an exact or approximate expression.
    Exact {
        /// The exact expression that serves as this [`StackVal`]'s true value.
        exact_expr: Expr<BigRational>,

        /// The approximation of `exact_expr`.
        approx_expr: Expr<f64>,

        /// The string rendering of `exact_expr`.
        exact_str: String,

        /// The string rendering of `approx_str`.
        approx_str: String,

        /// Whether to display the exact or approximate version of this value.
        is_approx: bool,
    },

    /// A stack value that is backed by an approximate expression, and can therefore only be
    /// displayed as an approximate expression.
    Approx {
        /// The approximate expression that serves as this [`StackVal`]'s true value.
        approx_expr: Expr<f64>,

        /// The string rendering of `approx_expr`.
        approx_str: String,
    },
}

impl StackVal {
    const fn approx_expr(&self) -> &Expr<f64> {
        match self {
            Self::Exact { approx_expr, .. } | Self::Approx { approx_expr, .. } => {
                approx_expr
            }
        }
    }

    // destructors cannot be evaluated at compile time
    #[allow(clippy::missing_const_for_fn)]
    fn into_approx_expr(self) -> Expr<f64> {
        match self {
            Self::Exact { approx_expr, .. } | Self::Approx { approx_expr, .. } => {
                approx_expr
            }
        }
    }

    fn apply_binary<ExactF, ApproxF, Return>(
        lhs: &Self,
        rhs: &Self,
        exact_f: ExactF,
        approx_f: ApproxF,
    ) -> Return
    where
        ExactF: Fn(&Expr<BigRational>, &Expr<BigRational>) -> Return,
        ApproxF: Fn(&Expr<f64>, &Expr<f64>) -> Return,
    {
        if let (
            Self::Exact {
                exact_expr: lhs_exact_expr,
                ..
            },
            Self::Exact {
                exact_expr: rhs_exact_expr,
                ..
            },
        ) = (lhs, rhs)
        {
            exact_f(lhs_exact_expr, rhs_exact_expr)
        } else {
            approx_f(lhs.approx_expr(), rhs.approx_expr())
        }
    }

    fn apply_unary<ExactF, ApproxF, Return>(&self, exact_f: ExactF, approx_f: ApproxF) -> Return
    where
        ExactF: Fn(&Expr<BigRational>) -> Return,
        ApproxF: Fn(&Expr<f64>) -> Return,
    {
        match self {
            Self::Exact { exact_expr, .. } => exact_f(exact_expr),
            Self::Approx { approx_expr, .. } => approx_f(approx_expr),
        }
    }
}

/// An expression, along with other data necessary for displaying it but not for doing math with it.
#[derive(Clone, PartialEq, Debug)]
pub struct StackItem {
    val: StackVal,
    radix: Radix,
}

impl StackItem {
    /// Create a new `StackItem` containing an exact expression and cache its rendered strings.
    #[must_use]
    pub fn new_exact(exact_expr: Expr<BigRational>, radix: Radix, config: &Config) -> Self {
        let approx_expr = exact_expr.clone().approx();
        let exact_str = exact_expr.display(radix, config);
        let approx_str = approx_expr.display(radix, config);
        Self {
            val: StackVal::Exact {
                exact_expr,
                approx_expr,
                exact_str,
                approx_str,
                is_approx: false,
            },
            radix,
        }
    }

    /// Create a new `StackItem` containing an approximate expression and cache its rendered string.
    #[must_use]
    pub fn new_approx(approx_expr: Expr<f64>, radix: Radix, config: &Config) -> Self {
        let approx_str = approx_expr.display(radix, config);
        Self {
            val: StackVal::Approx {
                approx_expr,
                approx_str,
            },
            radix,
        }
    }

    /// Update the cached strings in a stack item.
    pub fn rerender(&mut self, config: &Config) {
        match &mut self.val {
            StackVal::Exact {
                exact_expr,
                approx_expr,
                exact_str,
                approx_str,
                ..
            } => {
                *exact_str = exact_expr.display(self.radix, config);
                *approx_str = approx_expr.display(self.radix, config);
            }
            StackVal::Approx {
                approx_expr,
                approx_str,
            } => {
                *approx_str = approx_expr.display(self.radix, config);
            }
        }
    }
}

impl Display for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.val {
            StackVal::Exact {
                exact_str,
                approx_str,
                is_approx,
                ..
            } => {
                if *is_approx {
                    write!(f, "{approx_str}")
                } else {
                    write!(f, "{exact_str}")
                }
            }
            StackVal::Approx { approx_str, .. } => {
                write!(f, "{approx_str}")
            }
        }
    }
}

/// The global state of the calculator.
pub struct State<'a> {
    stack: Vec<StackItem>,

    /// A list of past stacks.
    history: Vec<Vec<StackItem>>,

    /// A list of stacks that have been undone.
    future: Vec<Vec<StackItem>>,

    /// The current text in the input field.
    input: String,

    /// The current text in the input field after "ᴇ".
    eex_input: Option<String>,

    /// The current text in the input field before "#".
    radix_input: Option<String>,

    /// The current local radix in the input field. If `self.radix_input` is empty or invalid, this should be `None`.
    input_radix: Option<Radix>,

    /// The soft error currently displaying on the modeline.
    err: Option<SoftError>,

    mode: Mode,

    /// The index of the selected item on the stack, or `None` if the input is selected.
    select_idx: Option<usize>,

    config: Config,

    stdout: StdoutLock<'a>,
}

impl<'a> State<'a> {
    const fn new(stdout: StdoutLock<'a>, config: Config) -> Self {
        Self {
            stack: Vec::new(),
            history: Vec::new(),
            future: Vec::new(),
            input: String::new(),
            eex_input: None,
            radix_input: None,
            input_radix: None,
            err: None,
            mode: Mode::Normal,
            select_idx: None,
            config,
            stdout,
        }
    }

    /// Return the index of the selected item, or the last item if none are selected.
    fn selected_or_last_idx(&self) -> Option<usize> {
        self.select_idx
            .map_or_else(|| self.stack.len().checked_sub(1), Some)
    }

    fn render(&mut self) -> Result<(), Error> {
        let (_, cy) = cursor::position().context("couldn't get cursor pos")?;
        self.stdout
            .queue(terminal::Clear(ClearType::CurrentLine))
            .context("couldn't clear the current line")?
            .queue(cursor::MoveTo(0, cy))
            .context("couldn't move the cursor to the start of the line")?;

        // the string which will be printed to the terminal, including formatting codes
        let mut s = String::new();
        // the apparent length of `s`, excluding formatting codes
        let mut len: usize = 0;
        // the midpoint of the selected expression, not as an index of `s`, but as an `x` coordinate of a terminal cell; `None` if no expression is selected
        let mut selected_pos = None;

        for i in 0..self.stack.len() {
            let stack_item = &self.stack[i];

            let expr_str = if option_env!("GUAC_DEBUG") == Some("true") {
                format!("{:?}", stack_item.val)
            } else {
                stack_item.to_string()
            };

            // if the current expression we're looking at is selected, assign to `selected_pos`
            if Some(i) == self.select_idx {
                selected_pos = Some(len + expr_str.len() / 2);
                s.push_str(&format!("{} ", expr_str.underline()));
            } else {
                s.push_str(&format!("{expr_str} "));
            }

            len += expr_str.len() + 1;
        }

        if self.mode == Mode::Pipe {
            s.push('|');
            len += 1;
        } else if self.mode == Mode::Cmd {
            s.push(':');
            len += 1;
        }

        // the position of the `#` in the input as a terminal column
        let mut hash_pos = None;
        if let Some(radix_input) = &self.radix_input {
            s.push_str(radix_input);
            s.push('#');
            len += radix_input.len();
            hash_pos = Some(len);
            len += 1;
        }

        let input = self.input.to_string();
        len += input.len();
        s.push_str(&input);

        if let Some(eex_input) = &self.eex_input {
            len += eex_input.len() + 1;
            s.push('ᴇ');
            s.push_str(eex_input);
        }

        let width = terminal::size().context("couldn't get terminal size")?.0 as usize;

        if len > (width - 1) {
            if let Some(pos) = selected_pos {
                // we have to crop `s` *around* the selected expr
                // the total length in chars of all the formatting escape codes in `s`
                let garbage = s.len().saturating_sub(len);
                let half_width = width / 2;
                // the leftmost index of `s` which will actually be displayed on the terminal
                let left = pos.saturating_sub(half_width);
                if let Some(i) = &mut hash_pos {
                    *i = i.saturating_sub(left);
                }

                // ditto for rightmost
                let right = (left + garbage + width - 1).clamp(0, s.len());

                s = s[left..right].to_string();
            } else {
                // no selected expr, so we can just crop off the left
                s.replace_range(0..len.saturating_sub(width - 1), "");
            }
        }

        print!("{}", s);

        if self.mode == Mode::Radix {
            if let Some(i) = hash_pos {
                self.stdout
                    .queue(cursor::MoveToColumn(i as u16 + 1))
                    .context("couldn't move cursor")?;
            }
        }

        if self.select_idx.is_some() && self.mode != Mode::Pipe && self.mode != Mode::Radix {
            self.stdout
                .queue(cursor::Hide)
                .context("couldn't hide cursor")?;
        } else {
            self.stdout
                .queue(cursor::Show)
                .context("couldn't show cursor")?;
        }

        self.stdout.flush().context("couldn't flush stdout")?;

        Ok(())
    }

    fn push_exact_expr(&mut self, expr: Expr<BigRational>, radix: Radix) {
        self.push_stack_item(StackItem::new_exact(expr, radix, &self.config));
    }

    fn push_approx_expr(&mut self, expr: Expr<f64>, radix: Radix) {
        self.push_stack_item(StackItem::new_approx(expr, radix, &self.config));
    }

    fn push_expr(&mut self, expr: StackExpr, radix: Radix) {
        match expr {
            StackExpr::Exact(expr) => self.push_exact_expr(expr, radix),
            StackExpr::Approx(expr) => self.push_approx_expr(expr, radix),
        }
    }

    fn push_stack_item(&mut self, stack_item: StackItem) {
        self.stack
            .insert(self.select_idx.unwrap_or(self.stack.len()), stack_item);

        if let Some(i) = &mut self.select_idx {
            *i += 1;
        }
    }

    fn drop(&mut self) {
        if let Some(i) = self.select_idx {
            self.stack.remove(i);

            if i == self.stack.len() {
                self.select_idx = None;
            }
        } else {
            self.stack.pop();
        }
    }

    fn parse_exact_expr(&self, s: &str) -> Result<Expr<BigRational>, SoftError> {
        self.input_radix
            .unwrap_or(self.config.radix)
            .parse_bigint(s)
            .map(|n| Expr::Num(BigRational::from(n)))
            .ok_or(SoftError::BadInput)
    }

    fn parse_approx_expr(&self, s: &str) -> Result<Expr<f64>, SoftError> {
        if self.input_radix.unwrap_or(self.config.radix) == Radix::DECIMAL {
            s.parse().map_err(|_| SoftError::BadInput).map(Expr::Num)
        } else {
            Err(SoftError::NonDecFloat)
        }
    }

    fn parse_expr(&self, s: &str) -> Result<StackExpr, SoftError> {
        if s.contains('.') {
            let e = self.parse_approx_expr(s)?;
            Ok(StackExpr::Approx(e))
        } else {
            let e = self.parse_exact_expr(s)?;
            Ok(StackExpr::Exact(e))
        }
    }

    fn push_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            if self.input_radix.is_none() {
                return None;
            } else if let Some(i) = self.selected_or_last_idx() {
                if let Some(x) = self.stack.get_mut(i) {
                    x.radix = self.input_radix.unwrap_or(self.config.radix);
                    x.rerender(&self.config);
                }

                self.input_radix = None;
                self.radix_input = None;
                self.reset_mode();

                return None;
            }
        }

        let radix = self.input_radix.unwrap_or(self.config.radix);

        // FIXME: wtf
        let eex = match self.eex_input.as_ref().map(|eex_input| {
            radix.parse_bigint(eex_input)
        }) {
            Some(None) => {
                self.err = Some(SoftError::BadEex);
                return None;
            }
            Some(other) => other,
            None => None,
        };

        match self.parse_expr(&self.input) {
            Ok(StackExpr::Approx(mut expr)) => {
                if let Some(eex) = eex {
                    if let Ok(eex) = i128::try_from(eex).map(|n| n as f64) {
                        expr *= Expr::from(radix).pow(Expr::Num(eex));
                    } else {
                        self.err = Some(SoftError::BigEex);
                    }
                }

                self.push_approx_expr(expr, radix);
            }
            Ok(StackExpr::Exact(mut expr)) => {
                if let Some(eex) = eex {
                    expr *= Expr::from(radix).pow(Expr::from(eex));
                }

                self.push_exact_expr(expr, radix);
            }
            Err(e) => {
                self.err = Some(e);
                return None;
            }
        };

        let prev_input = mem::take(&mut self.input);
        self.eex_input = None;
        self.radix_input = None;
        self.input_radix = None;
        self.reset_mode();

        Some(prev_input)
    }

    fn push_var(&mut self) {
        if !self.input.is_empty() {
            let radix = self.input_radix.unwrap_or(self.config.radix);
            let input = mem::take(&mut self.input);
            self.push_exact_expr(Expr::Var(input), radix);
        }
    }

    fn apply_binary<ExactF, ApproxF, ExactDomain, ApproxDomain>(
        &mut self,
        exact_f: ExactF,
        approx_f: ApproxF,
        are_in_domain_exact: ExactDomain,
        are_in_domain_approx: ApproxDomain,
    ) where
        ExactF: Fn(Expr<BigRational>, Expr<BigRational>) -> Expr<BigRational>,
        ApproxF: Fn(Expr<f64>, Expr<f64>) -> Expr<f64>,
        ExactDomain: Fn(&Expr<BigRational>, &Expr<BigRational>) -> Option<SoftError>,
        ApproxDomain: Fn(&Expr<f64>, &Expr<f64>) -> Option<SoftError>,
    {
        let prev_input = if self.select_idx.is_none() {
            self.push_input()
        } else {
            None
        };

        if self.stack.len() < 2 || self.select_idx == Some(0) {
            return;
        }

        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);

        if let Some(e) = StackVal::apply_binary(
            &self.stack[idx - 1].val,
            &self.stack[idx].val,
            are_in_domain_exact,
            are_in_domain_approx,
        ) {
            self.err = Some(e);

            if let Some(prev_input) = prev_input {
                self.input = prev_input;
            }

            return;
        }

        let x = self.stack.remove(idx - 1);
        let y = self.stack.remove(idx - 1);

        let radix = x.radix;

        let item = if let (
            StackVal::Exact {
                exact_expr: lhs_exact_expr,
                ..
            },
            StackVal::Exact {
                exact_expr: rhs_exact_expr,
                ..
            },
        ) = (x.val.clone(), y.val.clone())
        // TODO: ugh fix this when let-chains come out
        {
            StackItem::new_exact(exact_f(lhs_exact_expr, rhs_exact_expr), radix, &self.config)
        } else {
            StackItem::new_approx(
                approx_f(x.val.into_approx_expr(), y.val.into_approx_expr()),
                radix,
                &self.config,
            )
        };

        self.stack.insert(idx - 1, item);

        if let Some(i) = self.select_idx.as_mut() {
            *i -= 1;
        }
    }

    fn apply_unary<ExactF, ApproxF, ExactDomain, ApproxDomain>(
        &mut self,
        exact_f: ExactF,
        approx_f: ApproxF,
        is_in_domain_exact: ExactDomain,
        is_in_domain_approx: ApproxDomain,
    ) where
        ExactF: Fn(Expr<BigRational>) -> Expr<BigRational>,
        ApproxF: Fn(Expr<f64>) -> Expr<f64>,
        ExactDomain: Fn(&Expr<BigRational>) -> Option<SoftError>,
        ApproxDomain: Fn(&Expr<f64>) -> Option<SoftError>,
    {
        let prev_input = if self.select_idx.is_none() {
            self.push_input()
        } else {
            None
        };

        if self.stack.is_empty() {
            return;
        }

        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);

        if let Some(e) = self.stack[idx]
            .val
            .apply_unary(is_in_domain_exact, is_in_domain_approx)
        {
            self.err = Some(e);

            if let Some(prev_input) = prev_input {
                self.input = prev_input;
            }

            return;
        }

        let x = self.stack.remove(idx);

        let item = match x.val {
            StackVal::Exact { exact_expr, .. } => {
                StackItem::new_exact(exact_f(exact_expr), x.radix, &self.config)
            }
            StackVal::Approx { approx_expr, .. } => {
                StackItem::new_approx(approx_f(approx_expr), x.radix, &self.config)
            }
        };

        self.stack.insert(idx, item);
    }

    fn dup(&mut self) {
        if !self.stack.is_empty() {
            let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
            let e = self.stack[idx].clone();
            self.stack.insert(idx + 1, e);
            if let Some(i) = self.select_idx.as_mut() {
                *i += 1;
            }
        }
    }

    fn swap(&mut self) {
        if let Some(idx) = self.selected_or_last_idx() {
            if idx > 0 {
                self.stack.swap(idx - 1, idx);
            }
        }
    }

    fn toggle_approx(&mut self) {
        let idx = self.select_idx.unwrap_or(self.stack.len() - 1);
        if let Some(StackItem {
            val: StackVal::Exact { is_approx, .. },
            ..
        }) = self.stack.get_mut(idx)
        {
            *is_approx = !*is_approx;
        }
    }

    fn init_from_stdin(&mut self) {
        let stdin = stdin();

        if stdin.is_tty() {
            return;
        }

        let stdin = BufReader::new(stdin);
        let mut lines = stdin.lines();
        let mut idx: usize = 0;
        let mut bad_idxs = Vec::new();
        while let Some(Ok(line)) = lines.next() {
            idx += 1;
            let line: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            if let Ok(e) = self.parse_expr(&line) {
                self.push_expr(e, self.config.radix);
            } else {
                bad_idxs.push(idx);
            }
        }

        if !bad_idxs.is_empty() {
            eprintln!(
                "{}{} couldn't parse stdin (line{} {})",
                "info".bold().cyan(),
                ":".bold(),
                if bad_idxs.len() == 1 { "" } else { "s" },
                bad_idxs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    fn ev_loop(&mut self) -> Result<(), Error> {
        loop {
            self.err = None;

            // Read the next event from the terminal.
            if let Event::Key(kev) = event::read().context("couldn't get next terminal event")? {
                match self.handle_keypress(kev) {
                    Status::Render => {
                        self.write_modeline().context("couldn't write modeline")?;
                        self.render().context("couldn't render the state")?;
                        if let Some(old_stack) = self.history.last() {
                            if &self.stack != old_stack {
                                self.future.clear();
                                self.history.push(self.stack.clone());
                            }
                        } else {
                            self.future.clear();
                            self.history.push(self.stack.clone());
                        }
                    }
                    Status::Exit => {
                        break;
                    }
                    Status::Undo => {
                        if self.future.is_empty() {
                            self.history.pop();
                        }

                        if let Some(mut old_stack) = self.history.pop() {
                            mem::swap(&mut old_stack, &mut self.stack);
                            self.future.push(old_stack);
                        }

                        self.render().context("couldn't render the state")?;
                    }
                    Status::Redo => {
                        if let Some(mut new_stack) = self.future.pop() {
                            mem::swap(&mut new_stack, &mut self.stack);
                            self.history.push(new_stack);
                        }
                        self.render().context("couldn't render the state")?;
                    }
                    #[cfg(debug_assertions)]
                    Status::Debug => bail!("debug"),
                }
            }
        }

        Ok(())
    }

    fn start(&mut self) -> Result<(), Error> {
        terminal::enable_raw_mode().context("couldn't enable raw mode")?;

        let (cx, cy) = cursor::position().context("couldn't get cursor position")?;
        let (.., height) = terminal::size().context("couldn't get terminal size")?;

        // If the cursor is at the bottom of the screen, make room for one more line.
        if cy >= height - 1 {
            println!();
            self.stdout
                .execute(cursor::MoveTo(cx, cy - 1))
                .context("couldn't move cursor")?;
        }

        self.write_modeline().context("couldn't write modeline")?;
        self.render().context("couldn't render the state")?;

        self.ev_loop()?;

        Ok(())
    }
}

#[allow(unused_must_use)]
/// Try our best to clean up the terminal state; if too many errors happen, just print some newlines and call it good.
fn cleanup() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    if stdout.is_tty() {
        stdout.execute(cursor::Show);
        if terminal::disable_raw_mode().is_ok() {
            println!();
        } else {
            print!("\n\r\n\r");
        }
        stdout.execute(terminal::Clear(ClearType::CurrentLine));
    }
}

fn guac_interactive(force: bool) -> Result<(), Error> {
    let stdout = stdout();
    let stdout = stdout.lock();

    if !force {
        if !stdout.is_tty() {
            bail!("stdout is not a tty. use --force to run anyway.");
        } else if terminal::size().context("couldn't get terminal size")?.0 < 15 {
            bail!("terminal is too small. use --force to run anyway.")
        }
    }

    let config = Config::default();
    let mut state = State::new(stdout, config);

    state.init_from_stdin();

    state.start()?;

    Ok(())
}

fn go() -> Result<(), Error> {
    let args: Args = argh::from_env();

    match args.subc {
        Some(SubCommand::Keys(..)) => print!(include_str!("keys.txt")),
        Some(SubCommand::Version(..)) => {
            println!("guac v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            guac_interactive(args.force)?;
            cleanup();
        }
    }

    Ok(())
}

fn main() {
    let res = go();
    if let Err(e) = res {
        eprintln!("{}{} {e:#}", "guac error".bold().red(), ":".bold());
        exit(1);
    }
}
