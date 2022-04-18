use super::Expr;
use crate::util::are_unordered_eq;
use num::{BigRational, One};
use std::{ops::Add, fmt::Display};

/// A helper type to aid in the simplification of sums.
#[derive(Clone, PartialEq, Eq)]
pub struct Term {
    /// The coefficient of this term (`2` in `2*y*x^2`)
    pub coef: BigRational,
    /// The factors of this term (`y` and `x^2` in `2*y*x^2`)
    pub facs: Vec<Expr>,
}

impl Term {
    /// Convert this term into a corrected expression
    pub fn into_expr(self) -> Expr {
        let mut res = Expr::Product(self.coef, self.facs);
        res.correct();
        res
    }

    /// In lieu of the dependencies for the `num::One` trait, return the multiplicative identity of `Term`.
    pub fn one() -> Self {
        Self {
            coef: BigRational::one(),
            facs: Vec::new(),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().into_expr())
    }
}

impl Expr {
    /// Convert this expression into a term ready to add. e.g., turns `2*y*x^2` into `Term { coef: 2, factors: [y, x^2] }`
    pub fn into_term(self) -> Term {
        match self {
            Self::Num(coef) => Term { coef, facs: Vec::new() },
            Self::Product(coef, facs) => Term { coef, facs },
            other => Term { coef: One::one(), facs: vec![other] },
        }
    }

    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn into_terms(self) -> Vec<Term> {
        match self {
            Expr::Sum(ts) => ts,
            other => vec![other.into_term()],
        }
    }
}

impl Add for Expr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut self_terms = self.into_terms();
        'rhs: for rhs_term in rhs.into_terms() {
            for self_term in &mut self_terms {
                if are_unordered_eq(&self_term.facs, &rhs_term.facs) {
                    self_term.coef += rhs_term.coef;
                    continue 'rhs;
                }
            }

            self_terms.push(Term {
                coef: rhs_term.coef,
                facs: rhs_term.facs,
            });
        }

        let mut res = Self::Sum(self_terms);
        res.correct();
        res
    }
}
