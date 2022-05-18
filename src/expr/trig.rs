use std::{convert::TryInto, ops::Neg};

use num::{
    traits::{Inv, Pow},
    One, Signed, Zero,
};

use crate::config::AngleMeasure;

use super::{constant::Const, Expr};

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

impl Expr {
    /// Interpret the given expression as an angle in `measure`, and convert it to an angle in turns.
    #[must_use]
    pub fn into_turns(self, measure: AngleMeasure) -> Self {
        self / measure.full_turn()
    }

    /// Interpret the given expression as an angle in turns, and convert it to an angle in `measure`.
    #[must_use]
    pub fn turns_to(self, measure: AngleMeasure) -> Self {
        self * measure.full_turn()
    }

    /// Convert this expression from one angle measure into another.
    #[must_use]
    pub fn convert_angle(self, old_measure: AngleMeasure, new_measure: AngleMeasure) -> Self {
        self.into_turns(old_measure).turns_to(new_measure)
    }

    /// Take the sine of this expression as an angle in `measure`.
    #[must_use]
    pub fn generic_sin(self, measure: AngleMeasure) -> Self {
        let turns = self.clone().into_turns(measure) % Self::one();

        let onehalf = Self::from((1, 2));
        if turns.is_negative() {
            return self.neg().generic_sin(measure).neg();
        } else if turns >= onehalf {
            return (turns - onehalf)
                .turns_to(measure)
                .generic_sin(measure)
                .neg();
        } else if turns > Self::from((1, 4)) {
            return (onehalf - turns).turns_to(measure).generic_sin(measure);
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Self::zero(),
                (Ok(1), Ok(4)) => Self::one(),
                (Ok(1), Ok(8)) => Self::from(2).pow(Self::from((1, 2)).neg()),
                (Ok(1), Ok(6)) => Self::from(3).sqrt() / Self::from(2),
                (Ok(1), Ok(12)) => Self::from((1, 2)),
                _ => Self::Sin(Box::new(self), measure),
            },
            _ => Self::Sin(Box::new(self), measure),
        }
    }

    /// Take the cosine of this expression as an angle in `measure`.
    #[must_use]
    pub fn generic_cos(self, measure: AngleMeasure) -> Self {
        let turns = self.clone().into_turns(measure) % Self::one();

        let onehalf = Self::from((1, 2));
        if turns.is_negative() {
            return self.neg().generic_cos(measure);
        } else if turns >= onehalf {
            return (Self::one() - turns).turns_to(measure).generic_cos(measure);
        } else if turns > Self::from((1, 4)) {
            return (onehalf - turns)
                .turns_to(measure)
                .generic_cos(measure)
                .neg();
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Self::one(),
                (Ok(1), Ok(4)) => Self::zero(),
                (Ok(1), Ok(8)) => Self::from(2).pow(Self::from((1, 2)).neg()),
                (Ok(1), Ok(6)) => Self::from((1, 2)),
                (Ok(1), Ok(12)) => Self::from(3).sqrt() / Self::from(2),
                _ => Self::Cos(Box::new(self), measure),
            },
            _ => Self::Cos(Box::new(self), measure),
        }
    }

    /// Take the tangent of this expression as an angle in `measure`.
    #[must_use]
    pub fn generic_tan(self, measure: AngleMeasure) -> Self {
        let onehalf = Self::from((1, 2));

        let turns = self.clone().into_turns(measure) % onehalf.clone();
        if turns.is_negative() {
            return self.neg().generic_tan(measure);
        } else if turns > Self::from((1, 4)) {
            return (onehalf - turns)
                .turns_to(measure)
                .generic_tan(measure)
                .neg();
        }

        match turns {
            Self::Num(n) => match (n.numer().try_into(), n.denom().try_into()) {
                (Ok(0), ..) => Self::zero(),
                (Ok(1), Ok(24)) => Self::from(2) - Self::from(3).sqrt(),
                (Ok(1), Ok(12)) => Self::from(3).sqrt() / Self::from(3),
                (Ok(1), Ok(8)) => Self::one(),
                (Ok(1), Ok(6)) => Self::from(3).sqrt(),
                (Ok(5), Ok(24)) => Self::from(2) + Self::from(3).sqrt(),
                _ => Self::Tan(Box::new(self), measure),
            },
            _ => Self::Tan(Box::new(self), measure),
        }
    }

    /// Take the inverse sine of this expression in the current angle measure.
    #[must_use]
    pub fn asin(self, measure: AngleMeasure) -> Self {
        if self.is_negative() {
            return self.neg().asin(measure).neg();
        }

        if self.is_zero() {
            Self::zero().turns_to(measure)
        } else if self == Self::from((1, 2)) {
            Self::from((1, 12)).turns_to(measure)
        } else if self == Self::from(2).sqrt().inv() {
            Self::from((1, 8)).turns_to(measure)
        } else if self == Self::from(3).sqrt() / Self::from(2) {
            Self::from((1, 6)).turns_to(measure)
        } else if self.is_one() {
            Self::from((1, 4)).turns_to(measure)
        } else {
            Self::Asin(Box::new(self), measure)
        }
    }

    /// Take the inverse cosine of this expression in the current angle measure.
    #[must_use]
    pub fn acos(self, measure: AngleMeasure) -> Self {
        if self.is_negative() {
            return Self::one() - self.neg().asin(measure);
        }

        if self.is_zero() {
            Self::from((1, 4)).turns_to(measure)
        } else if self == Self::from((1, 2)) {
            Self::from((1, 6)).turns_to(measure)
        } else if self == Self::from(2).sqrt().inv() {
            Self::from((1, 8)).turns_to(measure)
        } else if self == Self::from(3).sqrt() / Self::from(2) {
            Self::from((1, 12)).turns_to(measure)
        } else if self.is_one() {
            Self::zero().turns_to(measure)
        } else {
            Self::Acos(Box::new(self), measure)
        }
    }

    /// Take the inverse tangent of this expression in the current angle measure.
    #[must_use]
    pub fn atan(self, measure: AngleMeasure) -> Self {
        if self.is_negative() {
            return self.neg().atan(measure).neg();
        }

        if self.is_zero() {
            Self::zero().turns_to(measure)
        } else if self == Self::from(2) - Self::from(3).sqrt() {
            Self::from((1, 24)).turns_to(measure)
        } else if self == Self::from(2) + Self::from(3).sqrt() {
            Self::from((5, 24)).turns_to(measure)
        } else if self == Self::from(3).sqrt().inv() {
            Self::from((1, 12)).turns_to(measure)
        } else if self == Self::from(3).sqrt() {
            Self::from((1, 6)).turns_to(measure)
        } else if self.is_one() {
            Self::from((1, 8)).turns_to(measure)
        } else {
            Self::Atan(Box::new(self), measure)
        }
    }
}
