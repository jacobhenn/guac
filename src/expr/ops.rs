use super::Expr;
use num::{
    rational::ParseRatioError,
    traits::{Inv, Pow},
    BigInt, BigRational, Num, One, Signed, Zero,
};
use std::{
    iter::{Product, Sum},
    ops::{Div, Neg, Rem, Sub},
};

impl Expr {
    /// Take the logarithm of self in base `base`. Perform obvious simplifications.
    #[must_use]
    pub fn log(self, base: Self) -> Self {
        if let Self::Power(b, e) = self {
            if base == *b {
                return *e;
            }

            return *b * base.log(*e);
        }

        Self::Log(Box::new(base), Box::new(self))
    }

    /// Take the square root of this expression.
    #[must_use]
    pub fn sqrt(self) -> Self {
        self.pow((1, 2).into())
    }
}

impl Zero for Expr {
    fn zero() -> Self {
        Self::Num(BigRational::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Num(n) => n.is_zero(),
            _ => false,
        }
    }

    fn set_zero(&mut self) {
        *self = Zero::zero();
    }
}

impl One for Expr {
    fn one() -> Self {
        Self::Num(BigRational::one())
    }
    fn is_one(&self) -> bool {
        match self {
            Self::Num(n) => n.is_one(),
            _ => false,
        }
    }
    fn set_one(&mut self) {
        *self = One::one();
    }
}

impl Sub for Expr {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (rhs * Self::Num(BigRational::from(BigInt::from(-1))))
    }
}

impl Div for Expr {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self * (rhs.pow(Self::Num(BigRational::from(BigInt::from(-1)))))
    }
}

impl Pow<Self> for Expr {
    type Output = Self;

    fn pow(mut self, mut rhs: Self) -> Self::Output {
        self.correct();
        rhs.correct();

        let mut out = match (self, rhs) {
            (Self::Num(b), Self::Num(e)) => {
                if e.is_integer() {
                    Self::Num(b.pow(e.numer()))
                } else {
                    Self::Power(Box::new(Self::Num(b)), Box::new(Self::Num(e)))
                }
            }
            (Self::Product(fs), rhs) => fs.into_iter().map(|f| f.pow(rhs.clone())).product(),
            (Self::Power(b, e), f) => Self::Power(b, Box::new(*e * f)),
            (b, e) => Self::Power(Box::new(b), Box::new(e)),
        };

        out.correct();
        out
    }
}

impl Neg for Expr {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self * (-1).into()
    }
}

impl Inv for Expr {
    type Output = Self;

    fn inv(self) -> Self::Output {
        self.pow((-1).into())
    }
}

impl Rem for Expr {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        if self < rhs {
            return self;
        }

        match (self, rhs) {
            (Self::Num(n), Self::Num(m)) => Self::Num(n % m),
            (lhs, rhs) => {
                let lhs_factors = lhs.into_factors();
                let rhs_factors = rhs.clone().into_factors();
                let outer_factors: Vec<Self> = rhs
                    .into_factors()
                    .into_iter()
                    .filter(|rf| lhs_factors.contains(rf))
                    .collect();
                let left: Self = lhs_factors
                    .into_iter()
                    .filter(|e| !outer_factors.contains(e))
                    .product();
                let right: Self = rhs_factors
                    .into_iter()
                    .filter(|e| !outer_factors.contains(e))
                    .product();
                outer_factors.into_iter().product::<Self>()
                    * match (left, right) {
                        (Self::Num(n), Self::Num(m)) => Self::Num(n % m),
                        (left, right) => Self::Mod(Box::new(left), Box::new(right)),
                    }
            }
        }
    }
}

impl Product for Expr {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(One::one(), |acc, i| acc * i)
    }
}

impl Sum for Expr {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Zero::zero(), |acc, i| acc + i)
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.clone()
            .to_f64()
            .ok()?
            .partial_cmp(&other.clone().to_f64().ok()?)
    }
}

impl Num for Expr {
    type FromStrRadixErr = ParseRatioError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::Num(BigRational::from_str_radix(str, radix)?))
    }
}

impl Signed for Expr {
    fn abs(&self) -> Self {
        if self.is_negative() {
            self.clone().neg()
        } else {
            self.clone()
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Self::zero()
        } else {
            self.clone() - other.clone()
        }
    }

    fn signum(&self) -> Self {
        if self.is_negative() {
            Self::one().neg()
        } else if self.is_positive() {
            Self::one()
        } else {
            Self::zero()
        }
    }

    fn is_positive(&self) -> bool {
        self.coefficient().map(|c| c.is_positive()).unwrap_or(true)
    }

    fn is_negative(&self) -> bool {
        !self.is_positive() && !self.is_zero()
    }
}
