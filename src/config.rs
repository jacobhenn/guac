use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use num::One;

use crate::{
    expr::{constant::Const, Expr},
    radix::Radix,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A unit of angle
pub enum AngleMeasure {
    /// 1/(2π) turn.
    Radian,

    /// 1 turn.
    Turn,

    /// 1/400 turn.
    Gradian,

    /// 1/360 turn.
    Degree,

    /// 1/21600 turn.
    Minute,

    /// 1/1296000 turn.
    Second,

    /// 1/256 turn.
    BinaryDegree,

    /// 1/24 turn.
    HourAngle,

    /// 1/32 turn.
    Point,

    /// 1/6400 turn.
    NatoMil,
}

impl AngleMeasure {
    /// Return how many of this angle measure make up a full turn.
    pub fn full_turn(self) -> Expr {
        match self {
            AngleMeasure::Radian => Expr::from_int(2) * Const::Pi.into(),
            AngleMeasure::Turn => Expr::one(),
            AngleMeasure::Gradian => Expr::from_int(400),
            AngleMeasure::Degree => Expr::from_int(360),
            AngleMeasure::Minute => Expr::from_int(21600),
            AngleMeasure::Second => Expr::from_int(1_296_000),
            AngleMeasure::BinaryDegree => Expr::from_int(256),
            AngleMeasure::HourAngle => Expr::from_int(24),
            AngleMeasure::Point => Expr::from_int(32),
            AngleMeasure::NatoMil => Expr::from_int(6400),
        }
    }
}

impl AngleMeasure {
    /// Return how many of this angle measure make up a full turn.
    pub const fn full_turn_f64(self) -> f64 {
        match self {
            AngleMeasure::Radian => std::f64::consts::TAU,
            AngleMeasure::Turn => 1.,
            AngleMeasure::Gradian => 400.,
            AngleMeasure::Degree => 360.,
            AngleMeasure::Minute => 21600.,
            AngleMeasure::Second => 1_296_000.,
            AngleMeasure::BinaryDegree => 256.,
            AngleMeasure::HourAngle => 24.,
            AngleMeasure::Point => 32.,
            AngleMeasure::NatoMil => 6400.,
        }
    }
}

impl Display for AngleMeasure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Radian => write!(f, "rad"),
            Self::Turn => write!(f, "turns"),
            Self::Gradian => write!(f, "grad"),
            Self::Degree => write!(f, "deg"),
            Self::Minute => write!(f, "min"),
            Self::Second => write!(f, "sec"),
            Self::BinaryDegree => write!(f, "bdeg"),
            Self::HourAngle => write!(f, "hour"),
            Self::Point => write!(f, "point"),
            Self::NatoMil => write!(f, "mil"),
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
