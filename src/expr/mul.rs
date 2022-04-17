use super::{add::Term, Expr};
use crate::util::unordered_eq;
use num::{BigRational, One};
use std::ops::Mul;

pub struct Factor {
    pub base: Expr,
    pub exp: Term,
}

impl Factor {
    pub fn expr(self) -> Expr {
        let mut expr = Expr::Power(Box::new(self.base), self.exp);
        expr.correct();
        expr
    }
}

impl Expr {
    pub fn factors(self) -> Vec<Factor> {
        match self {
            Self::Product(c, fs) => {
                let mut v = vec![Factor {
                    base: Expr::Num(c),
                    exp: Term::one(),
                }];
                for f in fs {
                    match f {
                        Self::Power(base, exp) => v.push(Factor { base: *base, exp }),
                        base => v.push(Factor {
                            base,
                            exp: Term::one(),
                        }),
                    }
                }
                v
            }
            Self::Power(base, exp) => vec![Factor { base: *base, exp }],
            base => vec![Factor {
                base,
                exp: Term::one(),
            }],
        }
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut self_factors = self.factors();
        'rhs_factor: for rhs_factor in rhs.factors() {
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
