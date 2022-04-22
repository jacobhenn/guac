use super::Expr;
use crate::util::are_unordered_eq;
use num::{traits::Pow, BigRational, One};
use std::ops::{Mul, MulAssign};

impl Expr {
    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    pub fn factors(&self) -> Vec<&Expr> {
        match self {
            Self::Product(fs) => fs.into_iter().collect(),
            other => vec![other],
        }
    }

    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    pub fn factors_mut(&mut self) -> Vec<&mut Expr> {
        match self {
            Self::Product(fs) => fs.into_iter().collect(),
            other => vec![other],
        }
    }

    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    pub fn into_factors(self) -> Vec<Expr> {
        match self {
            Self::Product(fs) => fs,
            other => vec![other],
        }
    }

    pub fn base(&self) -> Self {
        match self {
            Self::Power(b, ..) => *b.clone(),
            other => other.clone(),
        }
    }

    pub fn exponent(&self) -> Self {
        match self {
            Self::Power(_, e) => *e.clone(),
            _ => Self::one(),
        }
    }

    pub fn is_like_factor(&self, rhs: &Self) -> bool {
        self.base() == rhs.base() && self.exponent().is_like_term(&rhs.exponent())
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        let mut self_factors = self.into_factors();
        for rhs_factor in rhs.into_factors() {
            if let Some(self_factor) = self_factors
                .iter_mut()
                .find(|st| st.is_like_factor(&rhs_factor))
            {
                *self_factor = self_factor.base().pow(
                    self_factor
                        .exponent()
                        .combine_like_terms(rhs_factor.exponent()),
                )
            } else {
                self_factors.push(rhs_factor);
            }
        }

        let mut res = Self::Product(self_factors);
        res.correct();
        res
    }
}

impl MulAssign for Expr {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}
