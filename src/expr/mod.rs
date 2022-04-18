use self::add::Term;
use num::{BigRational, One, ToPrimitive, Zero};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    str::FromStr,
};

/// Implementation of `Add` for `Expr`, along with helper types and functions for that purpose.
pub mod add;

/// Implementation of `Mul` for `Expr`, along with helper types and functions for that purpose.
pub mod mul;

/// Implementation of various other number traits for `Expr`, along with helper types and functions for that purpose.
pub mod ops;

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

    /// The sine of another expression.
    // Sin(Box<Expr>),
    // Cos(Box<Expr>),
    // Tan(Box<Expr>),
    // SinInv(Box<Expr>),
    // CosInv(Box<Expr>),
    // TanInv(Box<Expr>),
    // Sinh(Box<Expr>),
    // Cosh(Box<Expr>),
    // Tanh(Box<Expr>),
    // SinhInv(Box<Expr>),
    // CoshInv(Box<Expr>),
    // TanhInv(Box<Expr>),
    /// A variable.
    Var(String),

    /// Euler's number: 2.718281.
    E,

    /// Full circle constant: 6.283185.
    Tau,
    // /// Imaginary unit
    // I,

    // // Physical Constants
    // C, // Speed of light in vacuum
    // H, // Planck constant
    // G, // Newtonian constant of gravitation
    // Na, // Avogadro constant
    // K, // Boltzmann constant
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
            Expr::E => Ok(std::f64::consts::E),
            Expr::Tau => Ok(std::f64::consts::TAU),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Sum(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|n| format!("({})", n))
                    .collect::<Vec<_>>()
                    .join("+"),
            ),
            Self::Product(c, fs) => write!(
                f,
                "{}*{}",
                c,
                fs.iter()
                    .map(|n| format!("({})", n))
                    .collect::<Vec<_>>()
                    .join("*"),
            ),
            Self::Power(b, e) => write!(f, "({})^({})", b, e),
            Self::Var(s) => write!(f, "{}", s),
            Self::Log(b, a) => write!(f, "log_({})({})", b, a),
            Self::E => write!(f, "e"),
            Self::Tau => write!(f, "τ"),
        }
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
