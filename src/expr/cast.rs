use super::{constant::Const, AngleMeasure::Radian, Expr};
use anyhow::{Context, Error};
use num::{BigInt, BigRational, ToPrimitive};
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

impl TryFrom<Expr> for f64 {
    type Error = ();

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Num(n) => n.to_f64().ok_or(()),
            Expr::Sum(ts) => ts.into_iter().map(Expr::to_f64).sum(),
            Expr::Product(fs) => fs
                .into_iter()
                .map(<Expr as TryInto<Self>>::try_into)
                .product::<Result<Self, _>>(),
            Expr::Power(b, e) => Ok(b.to_f64()?.powf(e.to_f64()?)),
            Expr::Log(b, a) => Ok(a.to_f64()?.log(b.to_f64()?)),
            Expr::Const(c) => Ok(c.into()),
            Expr::Mod(x, y) => Ok(x.to_f64()? % y.to_f64()?),
            Expr::Sin(x, m) => Ok(x.convert_angle(m, Radian).to_f64()?.sin()),
            _ => Err(()),
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
    fn from(i: (i128, i128)) -> Self {
        Self::Num(BigRational::from((i.0.into(), i.1.into())))
    }
}

/// Take a decimal number (like "5.64") and convert it to a rational number in lowest terms (in that case, 141/25).
pub fn parse_decimal_rational(s: &str) -> Option<BigRational> {
    let sep: Vec<_> = s.split('.').collect();

    if sep.len() == 2 {
        Some(
            sep[0].parse::<BigRational>().ok()?
                + sep[1].parse::<BigRational>().ok()?
                    / BigInt::from(10i32.pow(sep[1].len() as u32)),
        )
    } else {
        None
    }
}

impl FromStr for Expr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = s.parse::<BigRational>() {
            Ok(Self::Num(n))
        } else {
            Ok(Self::Num(
                parse_decimal_rational(s).context("couldn't parse from float")?,
            ))
        }
    }
}