use std::convert::TryInto;

use crate::config::AngleMeasure;

use self::add::Term;
use self::constant::Const;
use num::{BigRational, One, Zero, BigInt};

/// Implementation of `Add` for `Expr`, along with helper types and functions for that purpose.
pub mod add;

/// Implementation of `Mul` for `Expr`, along with helper types and functions for that purpose.
pub mod mul;

/// Implementation of various other number traits for `Expr`, along with helper types and functions for that purpose.
pub mod ops;

/// Implementation of `Display` for `Expr`, along with various other items for that purpose.
pub mod display;

/// Mathematical and physical constants.
pub mod constant;

/// Trigonometric functions.
pub mod trig;

/// Casting from expressions to other types and vice versa.
pub mod cast;

// /// Units. All of them.
// pub mod unit;

/// A general-purpose type to store algebraic expressions.
#[derive(Clone, PartialEq, Eq)]
pub enum Expr {
    /// A rational number.
    Num(BigRational),

    /// A sum of terms (pairs of rational and non-rational factors).
    Sum(Vec<Term>),

    /// A product of a rational coefficient and a number of non-rational expressions. It is not inherently guaranteed that the expressions will be non-rational, but `Expr::correct` will make them so.
    Product(BigRational, Vec<Expr>),

    /// One expression raised to the power of another.
    Power(Box<Expr>, Box<Expr>),

    /// The base-(first expression) logarithm of the second expression.
    Log(Box<Expr>, Box<Expr>),

    /// A variable.
    Var(String),

    /// A constant (`Const`).
    Const(Const),

    /// One expression modulo another.
    Mod(Box<Expr>, Box<Expr>),

    /// The sine of another expression in the given units.
    Sin(Box<Expr>, AngleMeasure),
}

impl Expr {
    /// Returns a floating-point approximation of the real number represented by this expression.
    pub fn to_f64(&self) -> Option<f64> {
        self.clone().try_into().ok()
    }

    /// Performs obvious and computationally inexpensive simplifications.
    pub fn correct(&mut self) {
        match self {
            Self::Num(_) => (),
            Self::Sum(ts) => {
                ts.iter_mut().for_each(|t| {
                    for f in &mut t.facs {
                        f.correct();
                    }
                });
                ts.retain(|t| !t.coef.is_zero());
                if ts.len() == 1 {
                    *self = ts[0].clone().into_expr();
                } else if ts.is_empty() {
                    self.set_zero();
                }
            }
            Self::Product(c, fs) => {
                for f in fs.iter_mut() {
                    f.correct();
                    if let Self::Num(n) = f {
                        *c *= n.clone();
                        *f = Self::Num(BigRational::one());
                    }
                }

                if c.is_zero() {
                    return *self = Self::zero();
                }

                fs.retain(|f| f != &Self::Num(BigRational::one()));

                if fs.is_empty() {
                    *self = Self::Num(c.clone());
                } else if c.is_one() && fs.len() == 1 {
                    *self = fs[0].clone();
                }
            }
            Self::Power(b, e) => {
                b.correct();
                e.correct();
                if e.is_one() {
                    *self = *b.clone();
                } else if e.is_zero() {
                    *self = One::one();
                }
            }
            _ => (),
        }
    }

    pub fn from_int<I>(i: I) -> Self where I: Into<i128> {
        Self::Num(BigRational::from(BigInt::from(i.into())))
    }
}

