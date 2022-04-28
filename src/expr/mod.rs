use std::convert::TryInto;

use crate::config::AngleMeasure;

use self::constant::Const;
use num::{BigInt, BigRational, One, Zero};

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
    Sum(Vec<Self>),

    /// A product of a rational coefficient and a number of non-rational expressions. It is not inherently guaranteed that the expressions will be non-rational, but `Expr::correct` will make them so.
    Product(Vec<Self>),

    /// One expression raised to the power of another.
    Power(Box<Self>, Box<Self>),

    /// The base-(first expression) logarithm of the second expression.
    Log(Box<Self>, Box<Self>),

    /// A variable.
    Var(String),

    /// A constant (`Const`).
    Const(Const),

    /// One expression modulo another.
    Mod(Box<Self>, Box<Self>),

    /// The sine of another expression in the given units.
    Sin(Box<Self>, AngleMeasure),

    /// The cosine of another expression in the given units.
    Cos(Box<Self>, AngleMeasure),

    /// The tangent of another expression in the given units.
    Tan(Box<Self>, AngleMeasure),
}

impl Expr {
    /// Is this expression a Num variant?
    pub const fn is_num(&self) -> bool {
        matches!(self, Self::Num(..))
    }

    /// Is this expression a Mod variant?
    pub const fn is_mod(&self) -> bool {
        matches!(self, Self::Mod(..))
    }

    /// Return the contents of this expression if it's a Num; if not, return None.
    #[allow(clippy::missing_const_for_fn)]
    pub fn num(&self) -> Option<&BigRational> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Return the contents of this expression if it's a Num; if not, return None.
    pub fn num_mut(&mut self) -> Option<&mut BigRational> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Return the contents of this expression if it's a Num; if not, return None.
    pub fn into_num(self) -> Option<BigRational> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Returns a floating-point approximation of the real number represented by this expression.
    pub fn to_f64(self) -> Result<f64, ()> {
        self.try_into()
    }

    /// Performs obvious and computationally inexpensive simplifications.
    pub fn correct(&mut self) {
        match self {
            Self::Sum(ts) => {
                for t in ts.iter_mut() {
                    t.correct();
                }
                ts.retain(|t| !t.is_zero());
                if ts.len() == 1 {
                    *self = ts[0].clone();
                } else if ts.is_empty() {
                    self.set_zero();
                }
            }
            Self::Product(fs) => {
                for f in fs.iter_mut() {
                    f.correct();
                }

                let c: BigRational = fs.iter_mut().filter_map(|f| f.num()).product();
                fs.retain(|f| !f.is_num());
                if c.is_zero() {
                    return self.set_zero();
                }

                if !c.is_one() {
                    fs.push(Self::Num(c));
                }

                if fs.is_empty() {
                    self.set_one();
                } else if fs.len() == 1 {
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

    /// If something can be an integer, it can be an Expr.
    pub fn from_int<I>(i: I) -> Self
    where
        I: Into<i128>,
    {
        Self::Num(BigRational::from(BigInt::from(i.into())))
    }
}
