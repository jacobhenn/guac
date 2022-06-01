use crate::config::AngleMeasure;

use super::{constant::Const, Expr};

use num::{traits::Pow, BigInt, BigRational, ToPrimitive};

/// Turn a `BigRational` into an `f64`.
pub fn frac2f64(n: BigRational) -> f64 {
    let max = BigInt::from(i128::MAX);

    let mut numer = n.numer().clone();
    let mut denom = n.denom().clone();

    while numer >= max && denom >= max {
        numer /= 2;
        denom /= 2;
    }

    let ni = numer.to_i128().unwrap();
    let di = denom.to_i128().unwrap();

    ni as f64 / di as f64
}

impl Expr {
    fn map_approx_binary<F, G>(x: Expr, y: Expr, f: F, g: G) -> Expr
    where
        F: Fn(f64, f64) -> f64,
        G: Fn(Expr, Expr) -> Expr,
    {
        let xa = x.approx();
        let ya = y.approx();

        if let (Expr::Num(m), Expr::Num(n)) = (xa.clone(), ya.clone()) {
            let (mf, nf) = (frac2f64(m), frac2f64(n));
            let rf = f(mf, nf);
            if let Some(r) = BigRational::from_float(rf) {
                return Expr::Num(r);
            }
        }

        g(xa, ya)
    }

    fn map_approx_unary<F, G>(x: Expr, f: F, g: G) -> Expr
    where
        F: Fn(f64) -> f64,
        G: Fn(Expr) -> Expr,
    {
        let xa = x.approx();

        if let Expr::Num(n) = xa.clone() {
            let nf = frac2f64(n);
            let rf = f(nf);
            if let Some(r) = BigRational::from_float(rf) {
                return Expr::Num(r);
            }
        }

        g(xa)
    }

    /// If something can be an integer, it can be an Expr.
    pub fn from_int<I>(i: I) -> Self
    where
        I: Into<i128>,
    {
        Self::Num(BigRational::from(BigInt::from(i.into())))
    }

    /// Reduce `self` by approximating. For example, turns
    pub fn approx(self) -> Self {
        match self {
            n @ Self::Num(_) | n @ Self::Var(_) | n @ Self::Const(_) => n,
            Self::Sum(ts) => ts.into_iter().map(|t| t.approx()).sum(),
            Self::Product(fs) => fs.into_iter().map(|f| f.approx()).product(),
            Self::Power(b, e) => Self::map_approx_binary(*b, *e, |b, e| b.powf(e), |b, e| b.pow(e)),
            Self::Log(b, a) => Self::map_approx_binary(*b, *a, |b, a| b.log(a), |b, a| b.log(a)),
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
                |x| convert_angle_f64(x.asin(), AngleMeasure::Radian, m),
                |x| x.acos(m),
            ),
            Self::Atan(x, m) => Self::map_approx_unary(
                *x,
                |x| convert_angle_f64(x.asin(), AngleMeasure::Radian, m),
                |x| x.atan(m),
            ),
        }
    }
}

/// Take an angle in `from` and convert it to an angle in `to`.
pub fn convert_angle_f64(x: f64, from: AngleMeasure, to: AngleMeasure) -> f64 {
    (x / from.full_turn_f64()) * to.full_turn_f64()
}

impl TryFrom<Expr> for f64 {
    type Error = ();

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        if let Expr::Num(n) = value.approx() {
            Ok(frac2f64(n))
        } else {
            Err(())
        }
    }
}

impl From<Const> for Expr {
    fn from(c: Const) -> Self {
        Self::Const(c)
    }
}

impl From<i128> for Expr {
    fn from(i: i128) -> Self {
        Self::Num(BigRational::from(BigInt::from(i)))
    }
}

impl From<(i128, i128)> for Expr {
    fn from((n, d): (i128, i128)) -> Self {
        Self::Num(BigRational::from((BigInt::from(n), BigInt::from(d))))
    }
}

/// Take a decimal number (like "5.64") and convert it to a rational number in lowest terms (in that case, 141/25).
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
