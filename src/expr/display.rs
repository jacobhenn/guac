use std::fmt::{self, Display, Formatter};

use num::{One, Signed};

use super::Expr;

impl Expr {
    /// The grouping priority of an expression represents its position in the order of operations; higher priority means further along in the order, i.e. addition has a higher priority than exponentiation.
    pub fn grouping_priority(&self) -> u8 {
        match self {
            Self::Num(n) => {
                if n.is_negative() {
                    4
                } else {
                    0
                }
            }
            Self::Power(..) => 1,
            Self::Product(..) | Self::Mod(..) => 2,
            Self::Sum(..) => 3,
            _ => 0,
        }
    }

    /// Use the grouping priority of `self` and `child` to decide wether or not to surround `child` in parens, then format it.
    pub fn format_child(&self, child: &Expr) -> String {
        if child.grouping_priority() > self.grouping_priority() {
            format!("({})", child)
        } else {
            format!("{}", child)
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Sum(ts) => write!(
                f,
                "{}",
                ts.iter()
                    .map(|t| self.format_child(&t))
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            Self::Product(fs) => {
                write!(
                    f,
                    "{}",
                    fs.iter()
                        .map(|f| self.format_child(&f))
                        .collect::<Vec<_>>()
                        .join("·")
                )
            }
            Self::Power(b, e) => write!(f, "{}^{}", self.format_child(b), self.format_child(e)),
            Self::Var(s) => write!(f, "{}", s),
            Self::Const(c) => write!(f, "{}", c),
            Self::Mod(x, y) => write!(f, "{} mod {}", self.format_child(x), self.format_child(y)),
            Self::Log(b, a) => write!(f, "log({})({})", b, a),
            Self::Sin(t, m) => write!(f, "sin({} {})", t, m),
        }
    }
}
