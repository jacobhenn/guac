use self::add::Term;
use self::constant::Const;
use num::{rational::ParseRatioError, BigRational, Num, One, ToPrimitive, Zero};
use std::{
    convert::{TryFrom, TryInto},
    ops::{Neg, Rem},
    str::FromStr,
};

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

    /// Take the logarithm of self in base `base`. Perform obvious simplifications.
    pub fn log(self, base: Expr) -> Expr {
        if let Expr::Power(b, e) = self {
            if base == *b {
                return *e;
            } else {
                return *b * base.log(*e);
            }
        }

        Expr::Log(Box::new(base), Box::new(self))
    }

    // pub fn sin(&mut self) {
    //     if self.is_zero() {
    //         self.set_zero();
    //     } else if {
    //     }
    // }
}

impl Neg for Expr {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::zero() - self
    }
}

impl Rem for Expr {
    type Output = Expr;

    fn rem(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Num for Expr {
    type FromStrRadixErr = ParseRatioError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::Num(BigRational::from_str_radix(str, radix)?))
    }
}

// impl Signed for Expr {
//     fn abs(&self) -> Self {
//         todo!()
//     }

//     fn abs_sub(&self, other: &Self) -> Self {
//         todo!()
//     }

//     fn signum(&self) -> Self {
//         match self {

//             other => BigInt::from_f64(other.to_f64().unwrap().signum())
//                 .unwrap()
//                 .into(),
//         }
//     }

//     fn is_positive(&self) -> bool {
//         todo!()
//     }

//     fn is_negative(&self) -> bool {
//         todo!()
//     }
// }

impl TryFrom<Expr> for f64 {
    type Error = ();

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Num(n) => n.to_f64().ok_or(()),
            Expr::Sum(ts) => ts
                .into_iter()
                .map(Term::into_expr)
                .map(<Expr as TryInto<f64>>::try_into)
                .sum(),
            Expr::Product(c, fs) => c.to_f64().ok_or(()).and_then(|x| {
                fs.into_iter()
                    .map(<Expr as TryInto<f64>>::try_into)
                    .product::<Result<f64, _>>()
                    .and_then(|p| Ok(x * p))
            }),
            Expr::Power(b, e) => Ok(b.to_f64().ok_or(())?.powf(e.to_f64().ok_or(())?)),
            Expr::Log(b, a) => Ok(a.to_f64().ok_or(())?.log(b.to_f64().ok_or(())?)),
            Expr::Const(c) => Ok(c.into()),
            // Expr::E => Ok(std::f64::consts::E),
            // Expr::Tau => Ok(std::f64::consts::TAU),
            _ => Err(()),
        }
    }
}

impl<T> From<T> for Expr
where
    T: Into<BigRational>,
{
    fn from(t: T) -> Self {
        Self::Num(t.into())
    }
}

impl FromStr for Expr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = s.parse::<BigRational>() {
            Ok(Self::Num(n))
        } else {
            Ok(Self::Num(
                BigRational::from_float(s.parse::<f64>().map_err(|_| ())?).ok_or(())?,
            ))
        }
    }
}
