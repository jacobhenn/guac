use std::ops::{Div, Mul, Neg};

use num::{
    traits::{Inv, Pow},
    One, Signed, Zero,
};

use crate::config::AngleMeasure;

use super::Expr;

impl<N> Expr<N> {
    /// Interpret the given expression as an angle in `measure`, and convert it to an angle in turns.
    #[must_use]
    pub fn into_turns(self, measure: AngleMeasure) -> Self
    where
        Self: Div<Output = Self> + From<i32> + Mul<Output = Self>,
    {
        self / measure.full_turn()
    }

    /// Interpret the given expression as an angle in turns, and convert it to an angle in `measure`.
    #[must_use]
    pub fn turns_to(self, measure: AngleMeasure) -> Self
    where
        Self: From<i32> + Mul<Output = Self>,
    {
        self * measure.full_turn()
    }

    /// Convert this expression from one angle measure into another.
    #[must_use]
    pub fn convert_angle(self, old_measure: AngleMeasure, new_measure: AngleMeasure) -> Self
    where
        Self: From<i32> + Mul<Output = Self> + Pow<Self, Output = Self> + PartialEq + One,
    {
        self.into_turns(old_measure).turns_to(new_measure)
    }

    /// Take the inverse sine of this expression in the current angle measure.
    // TODO: factor out these trait bounds
    #[must_use]
    pub fn asin(self, measure: AngleMeasure) -> Self
    where
        Self: Signed + From<(i32, i32)> + From<i32> + Pow<Self, Output = Self>,
    {
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
    pub fn acos(self, measure: AngleMeasure) -> Self
    where
        Self: Signed + From<(i32, i32)> + From<i32> + Pow<Self, Output = Self>,
    {
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
    pub fn atan(self, measure: AngleMeasure) -> Self
    where
        Self: Signed + From<(i32, i32)> + From<i32> + Pow<Self, Output = Self>,
    {
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

#[allow(clippy::trait_duplication_in_bounds)]
impl<N> Expr<N>
where
    Self: Clone
        + From<i32> // clippy thinks this is redundant; it isn't
        + Mul<Output = Self>
        + Div<Output = Self>
        + Pow<Self, Output = Self>
        + One
        + From<(i32, i32)>
        + Signed
        + PartialOrd,
{
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

        if turns.is_zero() {
            Self::zero()
        } else if turns == Self::from((1, 4)) {
            Self::one()
        } else if turns == Self::from((1, 8)) {
            Self::from(2).pow(Self::from((1, 2)).neg())
        } else if turns == Self::from((1, 6)) {
            Self::from(3).sqrt() / Self::from(2)
        } else if turns == Self::from((1, 12)) {
            Self::from((1, 2))
        } else {
            Self::Sin(Box::new(self), measure)
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

        if turns.is_zero() {
            Self::one()
        } else if turns == Self::from((1, 4)) {
            Self::zero()
        } else if turns == Self::from((1, 8)) {
            Self::from(2).pow(Self::from((1, 2)).neg())
        } else if turns == Self::from((1, 6)) {
            Self::from((1, 2))
        } else if turns == Self::from((1, 12)) {
            Self::from(3).sqrt() / Self::from(2)
        } else {
            Self::Cos(Box::new(self), measure)
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

        if turns.is_zero() {
            Self::zero()
        } else if turns == Self::from((1, 24)) {
            Self::from(2) - Self::from(3).sqrt()
        } else if turns == Self::from((1, 12)) {
            Self::from(3).sqrt() / Self::from(3)
        } else if turns == Self::from((1, 8)) {
            Self::one()
        } else if turns == Self::from((1, 6)) {
            Self::from(3).sqrt()
        } else if turns == Self::from((5, 24)) {
            Self::from(2) + Self::from(3).sqrt()
        } else {
            Self::Tan(Box::new(self), measure)
        }
    }
}
