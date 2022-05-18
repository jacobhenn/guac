use std::fmt::{Display, Formatter, self};

/// The configuration stored in `State` which will be read from a config file in the future.
pub struct Config {
    /// The angle measure that will be used for trig operations.
    pub angle_measure: AngleMeasure,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_measure: AngleMeasure::Radian,
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
            AngleMeasure::Radian => write!(f, "rad"),
            AngleMeasure::Turn => write!(f, "turns"),
            AngleMeasure::Gradian => write!(f, "grad"),
            AngleMeasure::Degree => write!(f, "deg"),
            AngleMeasure::Minute => write!(f, "min"),
            AngleMeasure::Second => write!(f, "sec"),
            AngleMeasure::BinaryDegree => write!(f, "bdeg"),
            AngleMeasure::HourAngle => write!(f, "hour"),
            AngleMeasure::Point => write!(f, "point"),
            AngleMeasure::NatoMil => write!(f, "mil"),
        }
    }
}
