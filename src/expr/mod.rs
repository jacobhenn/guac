use crate::{config::AngleMeasure, expr::constant::Const};

use std::iter::Product;

use num::{One, Zero};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<N> {
    /// A rational number.
    Num(N),

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

    /// The inverse sine of another expression in the given units.
    Asin(Box<Self>, AngleMeasure),

    /// The inverse cosine of another expression in the given units.
    Acos(Box<Self>, AngleMeasure),

    /// The inverse tangent of another expression in the given units.
    Atan(Box<Self>, AngleMeasure),
}

impl<N> Expr<N> {
    /// Are any of this expression's sub-expressions a variable?
    pub fn contains_var(&self) -> bool {
        match self {
            Self::Num(_) | Self::Const(_) => false,
            Self::Sum(xs) | Self::Product(xs) => xs.iter().any(Self::contains_var),
            Self::Power(x, y) | Self::Log(x, y) | Self::Mod(x, y) => {
                x.contains_var() || y.contains_var()
            }
            Self::Var(_) => true,
            Self::Sin(x, _)
            | Self::Cos(x, _)
            | Self::Tan(x, _)
            | Self::Asin(x, _)
            | Self::Acos(x, _)
            | Self::Atan(x, _) => x.contains_var(),
        }
    }

    /// How "big" is this expression in terms of sub-expressions?
    ///
    /// # Examples
    ///
    /// - The complexity of `2·x+5` is 3, one for each "leaf" of the expression tree.
    /// - The complexity of `sin(acos(tan(3)))` is 4, because even though there's only one "leaf"
    /// it's clearly more complex than the expression `3`.
    pub fn complexity(&self) -> u32 {
        match self {
            Self::Sum(ts) => ts.iter().map(Self::complexity).sum(),
            Self::Product(fs) => fs.iter().map(Self::complexity).sum(),
            Self::Power(x, y) => x.complexity() + y.complexity(),
            Self::Log(x, y) | Self::Mod(x, y) => x.complexity() + y.complexity() + 1,
            Self::Sin(x, _)
            | Self::Cos(x, _)
            | Self::Tan(x, _)
            | Self::Asin(x, _)
            | Self::Acos(x, _)
            | Self::Atan(x, _) => x.complexity() + 1,
            // This is not a catch-all, because I don't want it to silently catch new Expr
            // variants that don't have a complexity of 1.
            Self::Var(_) | Self::Const(_) | Self::Num(_) => 1,
        }
    }

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
    pub fn num(&self) -> Option<&N> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Return the contents of this expression if it's a Num; if not, return None.
    pub fn num_mut(&mut self) -> Option<&mut N> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Return the contents of this expression if it's a Num; if not, return None.
    // this function cannot be `const`, but clippy thinks it can
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_num(self) -> Option<N> {
        match self {
            Self::Num(n) => Some(n),
            _ => None,
        }
    }

    /// Performs obvious and computationally inexpensive simplifications.
    pub fn correct(&mut self)
    where
        N: Zero + One + Clone + for<'a> Product<&'a N> + PartialEq,
        Self: One + Zero,
    {
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

                let c: N = fs
                    .iter_mut()
                    .filter_map(|n| n.num() /* this can't be point-free :( */)
                    .product();
                fs.retain(|f| !f.is_num());
                if c.is_zero() {
                    return self.set_zero();
                }

                if !c.is_one() {
                    fs.insert(0, Self::Num(c));
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
}
