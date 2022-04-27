use super::Expr;
use num::{traits::{Pow, Inv}, One, BigRational};
use std::ops::{Mul, MulAssign};

impl Expr {
    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    pub fn factors(&self) -> Vec<&Self> {
        match self {
            Self::Product(fs) => fs.iter().collect(),
            other => vec![other],
        }
    }

    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    pub fn factors_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Self::Product(fs) => fs.iter_mut().collect(),
            other => vec![other],
        }
    }

    /// (Trivially) convert this expression into a list of its factors. **Does not actively factor expressions**. e.g., turns `2*x^2` into `[2, x^2]`, but turns `(2x+2)` into `[2x+2]`
    #[must_use]
    pub fn into_factors(self) -> Vec<Self> {
        match self {
            Self::Product(fs) => fs,
            other => vec![other],
        }
    }

    /// Return the base of this expression. e.g., x^2 -> x, x+5 -> x+5
    #[must_use]
    pub fn base(&self) -> Self {
        match self {
            Self::Num(n) if n.numer().is_one() => Self::Num(n.denom().clone().into()),
            Self::Power(b, ..) => *b.clone(),
            other => other.clone(),
        }
    }

    /// Return the exponent of this expression. e.g., x^2 -> 2, x+5 -> 1
    #[must_use]
    pub fn exponent(&self) -> Self {
        match self {
            Self::Num(n) if n.numer().is_one() => Self::from(-1),
            Self::Power(_, e) => *e.clone(),
            _ => Self::one(),
        }
    }

    /// Do these two terms have the same base and like terms for exponents?
    pub fn is_like_factor(&self, rhs: &Self) -> bool {
        self.base() == rhs.base() && self.exponent().is_like_term(&rhs.exponent())
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Self::Sum(ts) = self {
            return ts.into_iter().map(|t| t * rhs.clone()).sum();
        } else if let Self::Sum(ts) = rhs {
            return ts.into_iter().map(|t| t * self.clone()).sum();
        }

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
                );
            } else {
                self_factors.push(rhs_factor);
            }
        }

        let mut out = Self::Product(self_factors);
        out.correct();
        out
    }
}

impl MulAssign for Expr {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}
