use std::{fmt::{Display, Formatter, self}, str::FromStr};

use crate::radix::{self, Radix};

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
            radix: radix::DECIMAL,
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
