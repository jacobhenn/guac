use std::{
    fmt::{self, Display, Formatter},
    ops::Neg,
};

use num::{traits::Inv, One, Signed};

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
            Self::Product(..) => 2,
            Self::Sum(..) => 3,
            _ => 0,
        }
    }

    /// Use the grouping priority of `self` and `child` to decide wether or not to surround `child` in parens, then format it.
    pub fn format_child(&self, child: &Expr) -> String {
        if child.grouping_priority() > self.grouping_priority() || child.is_mod() {
            format!("({})", child)
        } else {
            format!("{}", child)
        }
    }

    /// Format this expression, but don't try to split products into a numerator and denominator.
    pub fn product_safe_format(&self, child: &Expr) -> String {
        match child {
            Self::Product(v) => {
                let str = v
                    .iter()
                    .map(|t| self.format_child(&t))
                    .collect::<Vec<_>>()
                    .join("·");

                if child.grouping_priority() > self.grouping_priority() {
                    format!("({})", str)
                } else {
                    str
                }
            }
            other => self.format_child(other),
        }
    }

    /// Does this expression have a negative exponent? Will also return true for fractions with a numerator of 1.
    pub fn has_pos_exp(&self) -> bool {
        match self {
            Self::Num(n) => !n.numer().is_one(),
            other => other.exponent().is_positive(),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Sum(ts) => {
                let (pos, neg): (Vec<&Expr>, Vec<&Expr>) = ts.iter().partition(|t| t.is_positive());

                write!(
                    f,
                    "{}",
                    pos.iter()
                        .map(|t| self.format_child(&t))
                        .collect::<Vec<_>>()
                        .join("+")
                )?;

                for n in neg {
                    write!(f, "-{}", self.format_child(&n.clone().neg()))?
                }

                Ok(())
            }
            Self::Product(fs) => {
                let (numer_vec, denom_vec): (Vec<&Expr>, Vec<&Expr>) =
                    fs.iter().partition(|f| f.has_pos_exp());

                let mut numer = Self::Product(numer_vec.into_iter().map(Clone::clone).collect());
                let mut denom =
                    Self::Product(denom_vec.into_iter().map(|f| f.clone().inv()).collect());
                numer.correct();
                denom.correct();

                write!(f, "{}", self.product_safe_format(&numer))?;
                if !denom.is_one() {
                    write!(f, "/{}", self.product_safe_format(&denom))?;
                }

                Ok(())
            }
            Self::Power(b, e) => {
                if **e == (1, 2).into() {
                    write!(f, "sqrt({})", b)
                } else if **e == (1, 3).into() {
                    write!(f, "cbrt({})", b)
                } else {
                    write!(f, "{}^{}", self.format_child(b), self.format_child(e))
                }
            }
            Self::Var(s) => write!(f, "{}", s),
            Self::Const(c) => write!(f, "{}", c),
            Self::Mod(x, y) => write!(f, "{} mod {}", self.format_child(x), self.format_child(y)),
            Self::Log(b, a) => write!(f, "log({})({})", b, a),
            Self::Sin(t, m) => write!(f, "sin({} {})", t, m),
        }
    }
}
