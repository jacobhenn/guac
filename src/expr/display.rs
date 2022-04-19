use std::fmt::{self, Display, Formatter};

use num::{One, Signed};

use super::Expr;

impl Expr {
    /// The grouping priority of an expression represents its position in the order of operations; higher priority means further along in the order, i.e. addition has a higher priority than exponentiation.
    pub fn grouping_priority(&self) -> u8 {
        match self {
            Self::Var(..) | Self::Const(..) | Self::Log(..) => 0,
            Self::Num(n) => {
                if n.is_negative() {
                    3
                } else {
                    0
                }
            }
            Self::Power(..) => 1,
            Self::Product(..) => 2,
            Self::Sum(..) => 3,
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
                    .map(|t| self.format_child(&t.clone().into_expr()))
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            Self::Product(c, fs) => {
                if !c.is_one() {
                    write!(f, "{}·", c)?;
                }
                write!(
                    f,
                    "{}",
                    fs.iter()
                        .map(|f| self.format_child(&f))
                        .collect::<Vec<_>>()
                        .join("·")
                )
            }
            Self::Power(b, e) => {
                write!(f, "{}", self.format_child(b))?;
                // match *e.clone() {
                //     Self::Num(n) => {
                //         if n.is_integer() {
                //             match n.to_i8().unwrap() {
                //                 2 => return write!(f, "²"),
                //                 3 => return write!(f, "³"),
                //                 4 => return write!(f, "⁴"),
                //                 5 => return write!(f, "⁵"),
                //                 6 => return write!(f, "⁶"),
                //                 7 => return write!(f, "⁷"),
                //                 8 => return write!(f, "⁸"),
                //                 9 => return write!(f, "⁹"),
                //                 -1 => return write!(f, "⁻¹"),
                //                 -2 => return write!(f, "⁻²"),
                //                 -3 => return write!(f, "⁻³"),
                //                 -4 => return write!(f, "⁻⁴"),
                //                 -5 => return write!(f, "⁻⁵"),
                //                 -6 => return write!(f, "⁻⁶"),
                //                 -7 => return write!(f, "⁻⁷"),
                //                 -8 => return write!(f, "⁻⁸"),
                //                 -9 => return write!(f, "⁻⁹"),
                //                 _ => (),
                //             };
                //         }
                //     },
                //     _ => (),
                // };
                write!(f, "^{}", self.format_child(e))
            }
            Self::Var(s) => write!(f, "{}", s),
            Self::Const(c) => write!(f, "{}", c),
            _ => todo!(),
        }
    }
}
