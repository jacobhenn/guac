use super::Expr;
use num::{traits::Pow, One, Zero};
use std::{
    iter::Product,
    ops::{Add, AddAssign, Mul, MulAssign},
};

impl<N> Expr<N> {
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
            // Self::Num(n) if n < BigRational::one() => self.inv(),
            Self::Power(b, ..) => *b,
            other => other,
        }
    }

    /// Return the exponent of this expression. e.g., x^2 -> 2, x+5 -> None
    pub const fn exponent(&self) -> Option<&Self> {
        match self {
            // Self::Num(n)
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
    pub fn into_exponent(self) -> Self
    where
        Self: One,
    {
        match self {
            // Self::Num(n) if n.numer().is_one() => Self::from(-1),
            Self::Power(_, e) => *e,
            _ => One::one(),
        }
    }

    /// Multiply two expressions. **Their exponents must be like terms, or this will be incorrect**.
    pub fn combine_like_factors(&mut self, rhs: Self)
    where
        N: One + Zero + PartialEq + AddAssign + Clone,
        Self: Clone + Pow<Self, Output = Self> + One + From<i32> + Add<Output = Self>,
    {
        if let Some(e) = self.exponent_mut() {
            e.combine_like_terms(rhs.into_exponent());
        } else {
            *self = self.clone().pow(Self::one() + rhs.into_exponent());
        }
    }

    /// Do these two terms have the same base and like terms for exponents?
    pub fn is_like_factor(&self, rhs: &Self) -> bool
    where
        N: PartialEq,
        Self: One + PartialEq + Clone,
    {
        self.clone().into_base() == rhs.clone().into_base()
            && self
                .exponent()
                .unwrap_or(&One::one())
                .is_like_term(rhs.exponent().unwrap_or(&One::one()))
    }

    /// Naively multiply two expressions, without performing any simplifications. Extends existing products instead of nesting.
    pub fn push_factor(&mut self, rhs: Self)
    where
        Self: Clone,
    {
        match self {
            Self::Product(fs) => fs.extend(rhs.into_factors()),
            other => {
                let mut v = vec![other.clone()];
                v.extend(rhs.into_factors());
                *other = Self::Product(v);
            }
        }
    }

    /// Multiply `self` by a single factor, but do not distribute over sums.
    pub fn mul_factor_nondistributing(&mut self, rhs: Self)
    where
        N: One + Zero + PartialEq + Clone + for<'a> Product<&'a N> + AddAssign,
        Self: Pow<Self, Output = Self> + From<i32>,
    {
        if let Some(factor) = self
            .factors_mut()
            .into_iter()
            .find(|x| x.is_like_factor(&rhs))
        {
            factor.combine_like_factors(rhs);
        } else {
            self.push_factor(rhs);
        }
    }
}

impl<N> Mul for Expr<N>
where
    N: One + Zero + PartialEq + Clone + for<'a> Product<&'a N> + AddAssign,
    Self: One + Zero + Clone + Pow<Self, Output = Self> + From<i32>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut out = Self::one();

        // the combined factors of `self` and `rhs` that `out` will be procedurally multiplied by
        let mut factors = self.into_factors();
        factors.append(&mut rhs.into_factors());

        // first, multiply everything that doesn't need distribution.
        // this cursed for loop is necessary because i'm mutating `factors` as i iterate through it
        let mut i = 0;
        while i < factors.len() {
            if let Self::Sum(_) = factors[i] {
                i += 1;
            } else {
                // read: is `factors[i]` a sum?
                let val = factors.remove(i);
                out.mul_factor_nondistributing(val);
            }
        }

        // `factors` now only contains sums, time to distribute
        for factor in factors {
            if let Self::Sum(terms) = factor {
                let mut new_res = Self::zero();
                for term in terms {
                    new_res += out.clone() * term;
                }
                out = new_res;
            }
        }

        out.correct();
        out
    }
}

impl<N> MulAssign for Expr<N>
where
    Self: Clone + Mul<Output = Self>,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}
