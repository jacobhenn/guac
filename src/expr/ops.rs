use num::{BigInt, BigRational, One, Zero};

use super::{Expr, add::Term};
use std::ops::{Add, Mul, Sub, Div};

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
        self * (rhs.pow(Term { coef: BigRational::from(BigInt::from(-1)), facs: vec![] }))
    }
}
