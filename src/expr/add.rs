use super::Expr;
use crate::util::unordered_eq;
use num::{BigRational, One, Zero};
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
    pub fn terms(self) -> Vec<Term> {
        match self {
            Expr::Num(coef) => vec![Term {
                coef,
                facs: Vec::new(),
            }],
            Expr::Sum(ts) => ts,
            Expr::Product(coef, facs) => vec![Term { coef, facs }],
            other => vec![Term {
                coef: BigRational::one(),
                facs: vec![other],
            }],
        }
    }
}

impl Add for Expr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut terms = Vec::new();
        let self_terms = self.terms();
        'rhs: for rhs_term in rhs.terms() {
            for self_term in &self_terms {
                if unordered_eq(&self_term.facs, &rhs_term.facs) {
                    terms.push(Term {
                        coef: self_term.coef.clone() + rhs_term.coef.clone(),
                        facs: self_term.facs.clone(),
                    });
                    continue 'rhs;
                }
            }

            terms.push(Term {
                coef: rhs_term.coef,
                facs: rhs_term.facs,
            });
        }

        let mut res = Self::Sum(terms);
        res.correct();
        res
    }
}
