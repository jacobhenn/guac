use super::{add::Term, Expr};
use crate::util::unordered_eq;
use num::{traits::Pow, BigRational, One};
use std::ops::Mul;

pub struct Factor {
    pub base: Expr,
    pub exp: Term,
}

impl Factor {
    pub fn expr(self) -> Expr {
        let mut expr = self.base.pow(self.exp.expr());
        expr.correct();
        expr
    }
}

impl Expr {
    pub fn into_factor(self) -> Factor {
        match self {
            Self::Power(base, exp) => Factor {
                base: *base,
                exp: exp.into_term(),
            },
            base => Factor {
                base,
                exp: Expr::one().into_term(),
            },
        }
    }

    pub fn into_factors(self) -> Vec<Factor> {
        match self {
            Self::Product(c, fs) => {
                let mut v = vec![Self::Num(c).into_factor()];
                for f in fs {
                    v.push(f.into_factor());
                }

                v
            }
            other => vec![other.into_factor()],
        }
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut self_factors = self.into_factors();
        'rhs_factor: for rhs_factor in rhs.into_factors() {
            for self_factor in &mut self_factors {
                if self_factor.base == rhs_factor.base
                    && unordered_eq(&self_factor.exp.facs, &rhs_factor.exp.facs)
                {
                    self_factor.exp.coef += rhs_factor.exp.coef;

                    continue 'rhs_factor;
                }
            }

            for self_factor in &mut self_factors {
                if self_factor.exp == rhs_factor.exp {
                    let mut base = Self::Product(
                        BigRational::one(),
                        vec![self_factor.base.clone(), rhs_factor.base.clone()],
                    );
                    base.correct();
                    self_factor.base = base;

                    continue 'rhs_factor;
                }
            }

            self_factors.push(rhs_factor);
        }

        let mut product = Self::Product(
            BigRational::one(),
            self_factors.into_iter().map(|f| f.expr()).collect(),
        );
        product.correct();
        product
    }
}
