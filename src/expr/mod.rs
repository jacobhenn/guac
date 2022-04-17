use self::add::Term;
use num::{
    traits::{Inv, Pow},
    BigRational, One, Signed, Zero,
};
use std::{
    fmt,
    str::FromStr,
};

mod add;
mod mul;
mod ops;

#[derive(Clone, PartialEq, Eq)]
pub enum Expr {
    Num(BigRational),

    // Algebraic functions
    Sum(Vec<Term>),
    Product(BigRational, Vec<Expr>),
    Power(Box<Expr>, Term),
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
    // SinhInv(Box<Expr>),
    // CoshInv(Box<Expr>),
    // TanhInv(Box<Expr>),
    Var(String),
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

impl Expr {
    // pub fn is_composite(&self) -> bool {
    //     match self {
    //         Self::Int(_) | Self::Ratio(_) => false,
    //         _ => true,
    //     }
    // }

    // pub fn is_simply_zero(&self) -> bool {
    //     match self {
    //         Self::Int(n) => n.is_zero(),
    //         Self::Ratio(n) => n.is_zero(),
    //         _ => false,
    //     }
    // }

    // pub fn is_num(&self) -> bool {
    //     match self {
    //         Self::Num(_) => true,
    //         _ => false,
    //     }
    // }

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
                    *self = ts[0].clone().expr();
                } else if ts.is_empty() {
                    self.set_zero();
                }
            }
            Self::Product(c, fs) => {
                for i in 0..fs.len() {
                    fs[i].correct();
                    match &fs[i] {
                        Self::Num(n) => {
                            *c *= n;
                            fs[i] = Self::Num(BigRational::one());
                        }
                        _ => (),
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
                if e.coef.is_one() && e.facs.is_empty() {
                    *self = *b.clone();
                } else if e.coef.is_zero() {
                    *self = Self::Num(BigRational::one());
                }
            }
            _ => (),
        }
    }
}

impl Pow<Expr> for Expr {
    type Output = Self;

    fn pow(mut self, mut rhs: Self) -> Self::Output {
        self.correct();
        rhs.correct();

        match (self, rhs) {
            (Self::Num(b), Self::Num(e)) => {
                if e.is_integer() {
                    if e.is_positive() {
                        Self::Num(b.pow(e.numer()))
                    } else {
                        Self::Num(b.pow(e.numer().abs()).inv())
                    }
                } else {
                    let mut res = Self::Power(Box::new(Self::Num(b)), Term { coef: e, facs: vec![] });
                    res.correct();
                    res
                }
            }
            (b, e) => {
                let exp = match e {
                    Self::Num(coef) => Term { coef, facs: vec![] },
                    Self::Product(coef, facs) => Term { coef, facs },
                    other => Term {
                        coef: One::one(),
                        facs: vec![other],
                    },
                };

                let mut res = Self::Power(Box::new(b), exp);
                res.correct();
                res
            }
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
