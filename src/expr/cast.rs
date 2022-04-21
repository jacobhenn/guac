use super::{add::Term, constant::Const, Expr, AngleMeasure::Radian};
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
            Expr::Sum(ts) => ts
                .into_iter()
                .map(Term::into_expr)
                .map(<Expr as TryInto<f64>>::try_into)
                .sum(),
            Expr::Product(c, fs) => c.to_f64().ok_or(()).and_then(|x| {
                fs.into_iter()
                    .map(<Expr as TryInto<f64>>::try_into)
                    .product::<Result<f64, _>>()
                    .map(|p| x * p)
            }),
            Expr::Power(b, e) => Ok(b.to_f64().ok_or(())?.powf(e.to_f64().ok_or(())?)),
            Expr::Log(b, a) => Ok(a.to_f64().ok_or(())?.log(b.to_f64().ok_or(())?)),
            Expr::Const(c) => Ok(c.into()),
            Expr::Mod(x, y) => Ok(x.to_f64().ok_or(())? % y.to_f64().ok_or(())?),
            Expr::Sin(x, m) => Ok(x.convert_angle(m, Radian).to_f64().ok_or(())?.sin()),
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

impl<T, U> From<(T, U)> for Expr
where
    T: Into<BigInt>,
    U: Into<BigInt>,
{
    fn from(i: (T, U)) -> Self {
        Self::Num(BigRational::from((i.0.into(), i.1.into())))
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
