use num::{BigRational, One, Zero};

use super::Expr;
use std::ops::{Add, Mul};

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

// impl One for Expr {
//     fn one() -> Self {
//         Self::Num(BigRational::one())
//     }
//     fn is_one(&self) -> bool {
//         match self {
//             Self::Num(n) => n.is_one(),
//             _ => false,
//         }
//     }
//     fn set_one(&mut self) {
//         *self = One::one();
//     }
// }
