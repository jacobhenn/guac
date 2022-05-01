use super::Expr;
use num::{traits::Pow, One, BigRational};
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
    pub fn into_base(self) -> Self {
        match self {
            Self::Num(n) if n.numer().is_one() => Self::Num(BigRational::from(n.denom().clone())),
            Self::Power(b, ..) => *b,
            other => other,
        }
    }

    /// Return the exponent of this expression. e.g., x^2 -> 2, x+5 -> None
    pub const fn exponent(&self) -> Option<&Self> {
        match self {
            Self::Power(_, e) => Some(e),
            _ => None,
        }
    }

    /// Return the exponent of this expression. e.g., x^2 -> 2, x+5 -> 1
    pub fn exponent_mut(&mut self) -> Option<&mut Self> {
        match self {
            Self::Power(_, e) => Some(e),
            _ => None,
        }
    }

    /// Return the exponent of this expression. e.g., x^2 -> 2, x+5 -> 1
    #[must_use]
    pub fn into_exponent(self) -> Self {
        match self {
            Self::Num(n) if n.numer().is_one() => Self::from(-1),
            Self::Power(_, e) => *e,
            _ => One::one(),
        }
    }

    /// Multiply two expressions. **Their exponents must be like terms, or this will be incorrect**.
    pub fn combine_like_factors(&mut self, rhs: Self) {
        if let Some(e) = self.exponent_mut() {
            e.combine_like_terms(rhs.into_exponent());
        } else {
            *self = self.clone().pow(Self::one() + rhs.into_exponent());
        }
    }

    /// Do these two terms have the same base and like terms for exponents?
    pub fn is_like_factor(&self, rhs: &Self) -> bool {
        self.clone().into_base() == rhs.clone().into_base()
            && self
                .exponent()
                .unwrap_or(&One::one())
                .is_like_term(rhs.exponent().unwrap_or(&One::one()))
    }

    /// Naively multiply two expressions, without performing any simplifications. Extends existing products instead of nesting.
    pub fn push_factor(&mut self, rhs: Self) {
        match self {
            Self::Product(fs) => fs.extend(rhs.into_factors()),
            other => {
                let mut v = vec![other.clone()];
                v.extend(rhs.into_factors());
                *other = Self::Product(v);
            }
        }
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign for Expr {
    fn mul_assign(&mut self, rhs: Self) {
        let self_factors = self.factors();
        let (like, unlike): (Vec<Self>, Vec<Self>) = rhs
            .into_terms()
            .into_iter()
            .partition(|t| self_factors.iter().any(|st| t.is_like_factor(st)));

        for factor in unlike {
            self.push_factor(factor);
        }

        let mut self_factors = self.factors_mut();
        for factor in like {
            if let Some(self_factor) = self_factors.iter_mut().find(|f| factor.is_like_factor(f)) {
                self_factor.combine_like_factors(factor);
            }
        }

        self.correct();
    }
}
