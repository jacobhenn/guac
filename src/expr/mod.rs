use num::{BigInt, BigRational};
use std::{
    fmt,
    ops::{Add, Div, Mul, Sub}, str::FromStr,
};

mod simplify;

#[derive(Clone)]
pub enum Expr {
    Int(BigInt),
    Ratio(BigRational),

    // Algebraic functions
    Sum(Vec<Expr>),
    Product(Vec<Expr>),
    Power(Box<Expr>, Box<Expr>),
    // Log(Box<Expr>, Box<Expr>),

    // // Trigonometric functions
    // Sin(Box<Expr>),
    // Cos(Box<Expr>),
    // Tan(Box<Expr>),
    // SinInv(Box<Expr>),
    // CosInv(Box<Expr>),
    // TanInv(Box<Expr>),
    // Sinh(Box<Expr>),
    // Cosh(Box<Expr>),
    // Tanh(Box<Expr>),
    // SinhI(Box<Expr>),
    // CoshI(Box<Expr>),
    // TanhI(Box<Expr>),

    // // Mathematical Constants
    // /// Euler's number: 2.718281
    // E,
    // /// Full circle constant: 6.283185
    // Tau,
    // /// Imaginary unit
    // I,

    // // Physical Constants
    // C, // Speed of light in vacuum
    // H, // Planck constant
    // G, // Newtonian constant of gravitation
    // Na, // Avogadro constant
    // K, // Boltzmann constant
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Ratio(n) => write!(f, "{}", n),
            Self::Sum(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|n| format!("({})", n))
                    .collect::<Vec<_>>()
                    .join("+"),
            ),
            Self::Product(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|n| format!("({})", n))
                    .collect::<Vec<_>>()
                    .join("*"),
            ),
            Self::Power(b, e) => write!(f, "({})^({})", b, e),
        }
    }
}

impl FromStr for Expr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Int(s.parse::<BigInt>()?))
    }
}

impl Add for Expr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut v = Vec::new();
        v.push(self);
        v.push(rhs);
        let mut raw = Self::Sum(v);
        raw.simplify();
        raw
    }
}

impl Sub for Expr {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut v = Vec::new();
        v.push(rhs);
        v.push(Self::Int(BigInt::from(-1)));
        let addend = Self::Product(v);
        let mut raw = self + addend;
        raw.simplify();
        raw
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut v = Vec::new();
        v.push(self);
        v.push(rhs);
        let mut raw = Self::Product(v);
        raw.simplify();
        raw
    }
}

impl Div for Expr {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let multiplicand = Self::Power(Box::new(rhs), Box::new(Self::Int(BigInt::from(-1))));
        let mut raw = self * multiplicand;
        raw.simplify();
        raw
    }
}

impl Expr {
    pub fn pow(self, rhs: Self) -> Self {
        Self::Power(Box::new(self), Box::new(rhs))
    }
}
