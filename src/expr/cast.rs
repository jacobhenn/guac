use crate::{
    config::{AngleMeasure, Config},
    radix::Radix,
    expr::Expr,
};

use num::{traits::Pow, BigInt, BigRational, ToPrimitive, rational::Ratio};

impl From<i32> for Expr<BigRational> {
    fn from(n: i32) -> Self {
        Self::Num(BigRational::from(BigInt::from(n)))
    }
}

impl From<(i32, i32)> for Expr<BigRational> {
    fn from((n, m): (i32, i32)) -> Self {
        Self::Num(BigRational::from((BigInt::from(n), BigInt::from(m))))
    }
}

impl From<BigInt> for Expr<BigRational> {
    fn from(n: BigInt) -> Self {
        Self::Num(BigRational::from(n))
    }
}

impl From<(i32, i32)> for Expr<f64> {
    fn from((n, m): (i32, i32)) -> Self {
        Self::Num(f64::from(n) / f64::from(m))
    }
}

macro_rules! impl_from_i32 {
    ( $(for $t:ty);+ ) => {
        $(
            impl From<i32> for Expr<$t> {
                fn from(n: i32) -> Self {
                    Self::Num(<$t>::from(n))
                }
            }
        )+
    }
}

impl_from_i32! {
    for i32; for i64; for i128; for f64; for Ratio<i32>
}

impl Expr<BigRational> {
    fn map_approx_binary<F, G>(x: Self, y: Self, f: F, g: G) -> Expr<f64>
    where
        F: Fn(f64, f64) -> f64,
        G: Fn(Expr<f64>, Expr<f64>) -> Expr<f64>,
    {
        let xa = x.approx();
        let ya = y.approx();

        if let (Expr::<f64>::Num(m), Expr::<f64>::Num(n)) = (xa.clone(), ya.clone()) {
            let (mf, nf) = (m.to_f64().unwrap(), n.to_f64().unwrap());
            let rf = f(mf, nf);
            if !rf.is_finite() {
                unreachable!(
                    "domain checks failed to detect non-finite result ({rf:?}) in binary operation {}",
                    g(Expr::Var(String::from("x")), Expr::Var(String::from("y")))
                        .display(Radix::DECIMAL, &Config::default()),
                );
            }

            return Expr::<f64>::Num(rf);
        }

        g(xa, ya)
    }

    fn map_approx_unary<F, G>(x: Self, f: F, g: G) -> Expr<f64>
    where
        F: Fn(f64) -> f64,
        G: Fn(Expr<f64>) -> Expr<f64>,
    {
        let xa = x.approx();

        if let Expr::<f64>::Num(n) = xa {
            let nf = n.to_f64().unwrap();
            let rf = f(nf);
            if !rf.is_finite() {
                unreachable!(
                    "domain checks failed to detect non-finite result ({rf:?}) in unary operation on {}",
                    g(Expr::Var(String::from("x")))
                        .display(Radix::DECIMAL, &Config::default()),
                );
            }

            return Expr::<f64>::Num(rf);
        }

        g(xa)
    }
}

impl Expr<BigRational> {
    /// Reduce `self` by approximating.
    ///
    /// # Panics
    ///
    /// Will panic if given an `Expr::Const(c)` such that `!f64::from(c).is_finite()`.
    #[must_use]
    pub fn approx(self) -> Expr<f64> {
        match self {
            // `<BigRational as ToPrimitive>::to_f64` cannot panic
            Self::Num(n) => Expr::<f64>::Num(n.to_f64().unwrap()),
            Self::Var(n) => Expr::<f64>::Var(n),
            Self::Const(c) => Expr::<f64>::Num(f64::from(c)),
            Self::Sum(ts) => ts.into_iter().map(Self::approx).sum(),
            Self::Product(fs) => fs.into_iter().map(Self::approx).product(),
            Self::Power(b, e) => Self::map_approx_binary(*b, *e, f64::powf, Expr::<f64>::pow),
            Self::Log(b, a) => Self::map_approx_binary(*b, *a, f64::log, Expr::<f64>::log),
            Self::Mod(n, d) => Self::map_approx_binary(*n, *d, |n, d| n % d, |n, d| n % d),
            Self::Sin(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x, m, AngleMeasure::Radian).sin(),
                |x| x.generic_sin(m),
            ),
            Self::Cos(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x, m, AngleMeasure::Radian).sin(),
                |x| x.generic_cos(m),
            ),
            Self::Tan(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x, m, AngleMeasure::Radian).sin(),
                |x| x.generic_tan(m),
            ),
            Self::Asin(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x.asin(), AngleMeasure::Radian, m),
                |x| x.asin(m),
            ),
            Self::Acos(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x.acos(), AngleMeasure::Radian, m),
                |x| x.acos(m),
            ),
            Self::Atan(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x.atan(), AngleMeasure::Radian, m),
                |x| x.atan(m),
            ),
        }
    }
}

/// Take an angle in `from` and convert it to an angle in `to`.
#[must_use]
pub fn convert_angle_f64(x: f64, from: AngleMeasure, to: AngleMeasure) -> f64 {
    (x / from.full_turn_f64()) * to.full_turn_f64()
}

/// Take a decimal number (like "5.64") and convert it to a rational number in lowest terms (in that case, 141/25).
// FIXME: this parsing could be way better
#[must_use]
pub fn parse_decimal_rational(s: &str) -> Option<BigRational> {
    let sep: Vec<_> = s.split('.').collect();

    if sep.len() == 2 {
        Some(
            sep[0].parse::<BigRational>().ok()?
                + sep[1].parse::<BigRational>().ok()? / BigInt::from(10u8).pow(sep[1].len() as u32),
        )
    } else {
        None
    }
}
