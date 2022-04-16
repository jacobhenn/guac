use std::fmt::Display;

use num::{BigRational, One, Zero};

use super::Expr;

#[derive(Clone, PartialEq, Eq)]
pub struct Term {
    pub coef: BigRational,
    pub facs: Vec<Expr>,
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}*{}",
            self.coef,
            self.facs
                .iter()
                .map(|n| format!("({})", n))
                .collect::<Vec<_>>()
                .join("*"),
        )
    }
}
