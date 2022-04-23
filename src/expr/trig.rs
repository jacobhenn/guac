use std::{convert::TryInto, ops::Neg};

use num::{One, Signed, Zero};

use crate::config::AngleMeasure;

use super::{constant::Const, Expr};

impl AngleMeasure {
    pub fn full_turn(&self) -> Expr {
        match self {
            AngleMeasure::Radian => Expr::from_int(2) * Const::Pi.into(),
            AngleMeasure::Turn => Expr::one(),
            AngleMeasure::Gradian => Expr::from_int(400),
            AngleMeasure::Degree => Expr::from_int(360),
            AngleMeasure::Minute => Expr::from_int(21600),
            AngleMeasure::Second => Expr::from_int(1296000),
            AngleMeasure::HalfTurn => Expr::from_int(2),
            AngleMeasure::Quadrant => Expr::from_int(4),
            AngleMeasure::Sextant => Expr::from_int(6),
            AngleMeasure::Hexacontade => Expr::from_int(60),
            AngleMeasure::BinaryDegree => Expr::from_int(256),
            AngleMeasure::HourAngle => Expr::from_int(24),
            AngleMeasure::Point => Expr::from_int(32),
            AngleMeasure::NatoMil => Expr::from_int(6400),
        }
    }
}

impl Expr {
    /// Interpret the given expression as an angle in `measure`, and convert it to an angle in turns.
    pub fn into_turns(self, measure: AngleMeasure) -> Expr {
        self / measure.full_turn()
    }

    /// Interpret the given expression as an angle in turns, and convert it to an angle in `measure`.
    pub fn turns_to(self, measure: AngleMeasure) -> Expr {
        self * measure.full_turn()
    }

    /// Convert this expression from one angle measure into another.
    pub fn convert_angle(self, old_measure: AngleMeasure, new_measure: AngleMeasure) -> Expr {
        self.into_turns(old_measure) * new_measure.full_turn()
    }

    /// Take the sine of this expression as an angle in `measure`.
    pub fn generic_sin(self, measure: AngleMeasure) -> Expr {
        let turns = self.clone().into_turns(measure) % Expr::one();
        print!("{}", turns);

        let onehalf: Expr = (1, 2).into();
        if turns.is_negative() && !turns.is_zero() {
            return self.neg().generic_sin(measure).neg();
        } else if turns >= onehalf {
            return (turns - onehalf)
                .turns_to(measure)
                .generic_sin(measure)
                .neg();
        } else if turns > (1, 4).into() {
            return (onehalf - turns).turns_to(measure).generic_sin(measure);
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Expr::zero(),
                (Ok(1), Ok(4)) => Expr::one(),
                (Ok(1), Ok(8)) => Expr::from(2).sqrt() / Expr::from(2),
                (Ok(1), Ok(6)) => Expr::from(3).sqrt() / Expr::from(2),
                (Ok(1), Ok(12)) => (1, 2).into(),
                _ => Self::Sin(Box::new(self), measure),
            },
            _ => Self::Sin(Box::new(self), measure),
        }
    }

    /// Take the cosine of this expression as an angle in `measure`.
    pub fn generic_cos(self, measure: AngleMeasure) -> Expr {
        let turns = self.clone().into_turns(measure) % Expr::one();

        let onehalf: Expr = (1, 2).into();
        if turns.is_negative() && !turns.is_zero() {
            return self.neg().generic_cos(measure);
        } else if turns >= onehalf {
            return (Expr::one() - turns).turns_to(measure).generic_cos(measure);
        } else if turns > (1, 4).into() {
            return (onehalf - turns)
                .turns_to(measure)
                .generic_cos(measure)
                .neg();
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Expr::one(),
                (Ok(1), Ok(4)) => Expr::zero(),
                (Ok(1), Ok(8)) => Expr::from(2).sqrt() / Expr::from(2),
                (Ok(1), Ok(6)) => (1, 2).into(),
                (Ok(1), Ok(12)) => Expr::from(3).sqrt() / Expr::from(2),
                _ => Self::Cos(Box::new(self), measure),
            },
            _ => Self::Cos(Box::new(self), measure),
        }
    }

    /// Take the tangent of this expression as an angle in `measure`.
    pub fn generic_tan(self, measure: AngleMeasure) -> Expr {
        let onehalf: Expr = (1, 2).into();

        let turns = self.clone().into_turns(measure) % onehalf.clone();
        if turns.is_negative() && !turns.is_zero() {
            return self.neg().generic_tan(measure);
        } else if turns > (1, 4).into() {
            return (onehalf - turns)
                .turns_to(measure)
                .generic_tan(measure)
                .neg();
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Expr::zero(),
                (Ok(1), Ok(24)) => Expr::from(2) - Expr::from(3).sqrt(),
                (Ok(1), Ok(12)) => Expr::from(3).sqrt() / Expr::from(3),
                (Ok(1), Ok(8)) => Expr::one(),
                (Ok(1), Ok(6)) => Expr::from(3).sqrt(),
                (Ok(5), Ok(24)) => Expr::from(2) + Expr::from(3).sqrt(),
                _ => Self::Tan(Box::new(self), measure),
            },
            _ => Self::Tan(Box::new(self), measure),
        }
    }
}
