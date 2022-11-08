// clippy thinks that AngleMeasure should be named `Self` because of #[derive(Display)].
// see https://github.com/rust-lang/rust-clippy/issues/9786
#![allow(clippy::use_self)]

use crate::{
    expr::{constant::Const, Expr},
    radix::Radix,
};

use std::{str::FromStr, ops::Mul};

use derive_more::Display;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// The configuration stored in `State` which will be read from a config file in the future.
pub struct Config {
    /// The angle measure that will be used for trig operations.
    pub angle_measure: AngleMeasure,

    /// The "default" radix in which numbers will be inputted or displayed.
    pub radix: Radix,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_measure: AngleMeasure::Radian,
            radix: Radix::DECIMAL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[cfg_attr(test, derive(Arbitrary))]
/// A unit of angle
pub enum AngleMeasure {
    /// 1/(2π) turn.
    #[display(fmt = "rad")]
    Radian,

    /// 1 turn.
    #[display(fmt = "turns")]
    Turn,

    /// 1/400 turn.
    #[display(fmt = "grad")]
    Gradian,

    /// 1/360 turn.
    #[display(fmt = "deg")]
    Degree,

    /// 1/21600 turn.
    #[display(fmt = "min")]
    Minute,

    /// 1/1296000 turn.
    #[display(fmt = "sec")]
    Second,

    /// 1/256 turn.
    #[display(fmt = "bdeg")]
    BinaryDegree,

    /// 1/24 turn.
    #[display(fmt = "hour")]
    HourAngle,

    /// 1/32 turn.
    #[display(fmt = "point")]
    Point,

    /// 1/6400 turn.
    #[display(fmt = "mil")]
    NatoMil,
}

impl AngleMeasure {
    /// Return how many of this angle measure make up a full turn.
    #[must_use]
    pub fn full_turn<N>(self) -> Expr<N>
    where
        Expr<N>: Mul<Output = Expr<N>> + From<i32>,
    {
        match self {
            Self::Radian => Expr::from(2) * Expr::Const(Const::Pi),
            other => Expr::from(match other {
                Self::Turn => 1,
                Self::Gradian => 400,
                Self::Degree => 360,
                Self::Minute => 21600,
                Self::Second => 1_296_000,
                Self::BinaryDegree => 256,
                Self::HourAngle => 24,
                Self::Point => 32,
                Self::NatoMil => 6400,
                Self::Radian => unreachable!(),
            }),
        }
    }

    /// Return how many of this angle measure make up a full turn.
    #[must_use]
    pub const fn full_turn_f64(self) -> f64 {
        match self {
            Self::Radian => std::f64::consts::TAU,
            Self::Turn => 1.0,
            Self::Gradian => 400.0,
            Self::Degree => 360.0,
            Self::Minute => 21600.0,
            Self::Second => 1_296_000.0,
            Self::BinaryDegree => 256.0,
            Self::HourAngle => 24.0,
            Self::Point => 32.0,
            Self::NatoMil => 6400.0,
        }
    }
}

impl FromStr for AngleMeasure {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rad" => Ok(Self::Radian),
            "turns" => Ok(Self::Turn),
            "grad" => Ok(Self::Gradian),
            "deg" => Ok(Self::Degree),
            "min" => Ok(Self::Minute),
            "sec" => Ok(Self::Second),
            "bdeg" => Ok(Self::BinaryDegree),
            "hour" => Ok(Self::HourAngle),
            "point" => Ok(Self::Point),
            "mil" => Ok(Self::NatoMil),
            _ => Err(()),
        }
    }
}
