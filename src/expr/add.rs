use super::Expr;
use crate::util::unordered_eq;
use num::{BigRational, One};
use std::{ops::Add, fmt::Display};

#[derive(Clone, PartialEq, Eq)]
pub struct Term {
    pub coef: BigRational,
    pub facs: Vec<Expr>,
}

impl Term {
    pub fn expr(self) -> Expr {
        let mut res = Expr::Product(self.coef, self.facs);
        res.correct();
        res
    }

    pub fn one() -> Self {
        Self {
            coef: BigRational::one(),
            facs: Vec::new(),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().expr())
    }
}

impl Expr {
    pub fn into_term(self) -> Term {
        match self {
            Self::Num(coef) => Term { coef, facs: Vec::new() },
            Self::Product(coef, facs) => Term { coef, facs },
            other => Term { coef: One::one(), facs: vec![other] },
        }
    }

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
                if unordered_eq(&self_term.facs, &rhs_term.facs) {
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
