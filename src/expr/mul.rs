use std::ops::Mul;

use super::Expr;

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // let mut factors = Vec::new();
        todo!()
    }
}
