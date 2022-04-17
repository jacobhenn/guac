use num::{
    traits::{Inv, Pow},
    BigInt, BigRational, One, Signed, Zero,
};

use super::Expr;
use std::ops::{Div, Sub};

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

impl Pow<Expr> for Expr {
    type Output = Self;

    fn pow(mut self, mut rhs: Self) -> Self::Output {
        self.correct();
        rhs.correct();

        match (self, rhs) {
            (Self::Num(b), Self::Num(e)) => {
                if e.is_integer() {
                    if e.is_positive() {
                        Self::Num(b.pow(e.numer()))
                    } else {
                        Self::Num(b.pow(e.numer().abs()).inv())
                    }
                } else {
                    let mut res = Self::Power(Box::new(Self::Num(b)), Box::new(Self::Num(e)));
                    res.correct();
                    res
                }
            }
            (Self::Power(b, e), f) => Self::Power(b, Box::new(*e * f)),
            (b, e) => {
                let mut res = Self::Power(Box::new(b), Box::new(e));
                res.correct();
                res
            }
        }
    }
}
