#![allow(dead_code)]

use std::fmt::{Display, Formatter, self};

pub struct Config {
    pub angle_measure: AngleMeasure,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_measure: AngleMeasure::Radian,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AngleMeasure {
    Radian,
    Turn,
    Gradian,
    Degree,
    Minute,
    Second,
    HalfTurn,
    Quadrant,
    Sextant,
    Hexacontade,
    BinaryDegree,
    HourAngle,
    Point,
    NatoMil,
}

impl Display for AngleMeasure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AngleMeasure::Radian => write!(f, "rad"),
            AngleMeasure::Turn => write!(f, "turn"),
            AngleMeasure::Gradian => write!(f, "grad"),
            AngleMeasure::Degree => write!(f, "deg"),
            AngleMeasure::Minute => write!(f, "min"),
            AngleMeasure::Second => write!(f, "sec"),
            AngleMeasure::HalfTurn => write!(f, "mulπ"),
            AngleMeasure::Quadrant => write!(f, "quad"),
            AngleMeasure::Sextant => write!(f, "sext"),
            AngleMeasure::Hexacontade => write!(f, "hexacontade"),
            AngleMeasure::BinaryDegree => write!(f, "bdeg"),
            AngleMeasure::HourAngle => write!(f, "hour"),
            AngleMeasure::Point => write!(f, "point"),
            AngleMeasure::NatoMil => write!(f, "mil"),
        }
    }
}
