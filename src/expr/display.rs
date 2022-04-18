use std::fmt::{self, Display, Formatter};

use num::One;

use super::Expr;

impl Expr {
    pub fn grouping_priority(&self) -> u8 {
        match self {
            Self::Num(..) | Self::Var(..) | Self::Const(..) | Self::Log(..) => 0,
            Self::Power(..) => 1,
            Self::Product(..) => 2,
            Self::Sum(..) => 3,
        }
    }

    pub fn display_child(&self, child: &Expr) -> String {
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
                    .map(|t| self.display_child(&t.clone().into_expr()))
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
                        .map(|f| self.display_child(&f))
                        .collect::<Vec<_>>()
                        .join("+")
                )
            }
            Self::Power(b, e) => {
                write!(f, "{}", self.display_child(b))?;
                write!(f, "^")?;
                write!(f, "{}", self.display_child(e))
            }
            Self::Var(s) => write!(f, "{}", s),
            _ => todo!(),
        }
    }
}
