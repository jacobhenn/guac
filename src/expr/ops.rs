use super::Expr;
use num::{
    traits::{Inv, Pow},
    BigRational, Num, One, Signed, Zero, BigInt,
};
use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
use num::integer::Roots;

impl<N> Expr<N> {
    /// Take the logarithm of self in base `base`. Perform obvious simplifications.
    #[must_use]
    pub fn log(self, base: Self) -> Self
    where
        N: PartialEq,
        Self: Mul<Output = Self>,
    {
        match (self, base) {
            (Self::Power(b, e), base) => {
                if base == *b {
                    *e
                } else {
                    *b * base.log(*e)
                }
            }
            (other, base) => Self::Log(Box::new(base), Box::new(other)),
        }
    }
}

impl<N> Expr<N>
where
    Self: Pow<Self, Output = Self> + From<(i32, i32)>,
{
    /// Take the square root of this expression.
    #[must_use]
    pub fn sqrt(self) -> Self {
        self.pow(Self::from((1, 2)))
    }
}

impl<N> Zero for Expr<N>
where
    Self: Add<Output = Self>,
    N: Zero,
{
    fn zero() -> Self {
        Self::Num(N::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Num(n) => n.is_zero(),
            _ => false,
        }
    }

    fn set_zero(&mut self) {
        *self = Zero::zero();
    }
}

impl<N> One for Expr<N>
where
    N: One + Zero + PartialEq + Clone + for<'a> Product<&'a N> + AddAssign,
    Self: Pow<Self, Output = Self> + From<i32>,
{
    fn one() -> Self {
        Self::Num(N::one())
    }
    fn is_one(&self) -> bool {
        match self {
            Self::Num(n) => n.is_one(),
            _ => false,
        }
    }
    fn set_one(&mut self) {
        *self = One::one();
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<N> Sub for Expr<N>
where
    Self: Add<Output = Self> + Neg<Output = Self>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs.neg()
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<N> SubAssign for Expr<N>
where
    Self: Neg<Output = Self> + AddAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs.neg();
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<N> Div for Expr<N>
where
    Self: Mul<Output = Self> + Inv<Output = Self> + PartialEq + One,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // TODO: this is disgusting
        if self == rhs {
            return Self::one();
        }

        self * rhs.inv()
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<N> DivAssign for Expr<N>
where
    Self: MulAssign + Inv<Output = Self>,
{
    fn div_assign(&mut self, rhs: Self) {
        *self *= rhs.inv();
    }
}

trait NumPow: Sized {
    fn pow(self, rhs: Self) -> Expr<Self>;
}

fn try_perfect_nth_root(lhs: &BigRational, rhs: &BigInt) -> Option<BigInt> {
    if !lhs.is_integer() {
        return None;
    }

    u32::try_from(rhs).ok().and_then(|rhs| {
        let lhs_int = lhs.to_integer();
        let root = lhs_int.nth_root(rhs);
        (root.clone().pow(rhs) == lhs_int).then_some(root)
    })
}

#[cfg(test)]
proptest! {
    #[test]
    fn test_is_rootable_by(
        m in 2..=8i32,
        n in 0..=u32::MAX.sqrt().sqrt(),
    ) {
        let rm = BigInt::from(m);
        let rn = BigRational::from(BigInt::from(n));
        assert!(try_perfect_nth_root(&<BigRational as Pow<_>>::pow(rn, &rm), &rm).is_some());
    }
}

impl NumPow for BigRational {
    fn pow(self, rhs: Self) -> Expr<Self> {
        if rhs.is_integer() {
            Expr::Num(<Self as Pow<_>>::pow(self, rhs.numer()))
        } else if let Some(root) = try_perfect_nth_root(&self, rhs.denom()) {
            Expr::Num(BigRational::from(root))
        } else {
            Expr::Power(Box::new(Expr::Num(self)), Box::new(Expr::Num(rhs)))
        }
    }
}

impl NumPow for i32 {
    fn pow(self, rhs: Self) -> Expr<Self> {
        if rhs.is_positive() {
            Expr::Num(<Self as Pow<_>>::pow(self, rhs.unsigned_abs()))
        } else {
            Expr::Power(Box::new(Expr::Num(self)), Box::new(Expr::Num(rhs)))
        }
    }
}

macro_rules! impl_num_pow {
    ( $(for $t:ty);+ ) => {
        $(
            impl NumPow for $t {
                fn pow(self, rhs: Self) -> Expr<Self> {
                    Expr::Num(<Self as Pow<_>>::pow(self, rhs))
                }
            }
        )+
    }
}

impl_num_pow! {
    for f32; for f64
}

impl<N> Pow<Self> for Expr<N>
where
    N: NumPow + Zero + One + Clone + for<'a> Product<&'a N> + PartialEq + AddAssign,
    Self: From<i32>
{
    type Output = Self;

    fn pow(mut self, mut rhs: Self) -> Self::Output {
        self.correct();
        // if self.is_one() {
        //     return self;
        // }

        rhs.correct();

        let mut out = match (self, rhs) {
            (Self::Num(b), Self::Num(e)) => <N as NumPow>::pow(b, e),
            (Self::Product(fs), rhs) => fs.into_iter().map(|f| f.pow(rhs.clone())).product(),
            (Self::Power(b, e), f) => Self::Power(b, Box::new(*e * f)),
            (b, e) => Self::Power(Box::new(b), Box::new(e)),
        };

        out.correct();
        out
    }
}

impl<N> Neg for Expr<N>
where
    Self: Mul<Output = Self> + From<i32>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self * Self::from(-1)
    }
}

impl<N> Inv for Expr<N>
where
    Self: Pow<Self, Output = Self> + From<i32>,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        self.pow(Self::from(-1))
    }
}

impl<N> Rem for Expr<N>
where
    N: Rem<Output = N>,
    Self: PartialOrd + Clone + Product + Mul<Output = Self>,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        if self < rhs {
            return self;
        }

        match (self, rhs) {
            (Self::Num(n), Self::Num(m)) => Self::Num(n % m),
            (lhs, rhs) => {
                let lhs_factors = lhs.into_factors();
                let rhs_factors = rhs.clone().into_factors();
                let outer_factors: Vec<Self> = rhs
                    .into_factors()
                    .into_iter()
                    .filter(|rf| lhs_factors.contains(rf))
                    .collect();
                let left: Self = lhs_factors
                    .into_iter()
                    .filter(|e| !outer_factors.contains(e))
                    .product();
                let right: Self = rhs_factors
                    .into_iter()
                    .filter(|e| !outer_factors.contains(e))
                    .product();
                outer_factors.into_iter().product::<Self>()
                    * match (left, right) {
                        (Self::Num(n), Self::Num(m)) => Self::Num(n % m),
                        (left, right) => Self::Mod(Box::new(left), Box::new(right)),
                    }
            }
        }
    }
}

impl<N> RemAssign for Expr<N>
where
    Self: Rem<Output = Self>,
    N: Clone,
{
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.clone() % rhs;
    }
}

impl<N> Product for Expr<N>
where
    Self: One,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, i| acc * i)
    }
}

impl<N> Sum for Expr<N>
where
    Self: Zero,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, i| acc + i)
    }
}

impl<N> PartialOrd for Expr<N>
where
    N: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.num()
            .and_then(|n| other.num().and_then(|m| n.partial_cmp(m)))
    }
}

impl<N> Num for Expr<N>
where
    N: Num + Clone + for<'a> Product<&'a N> + AddAssign,
    Self: Pow<Self, Output = Self> + From<i32> + Rem<Output = Self>,
{
    type FromStrRadixErr = N::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::Num(N::from_str_radix(str, radix)?))
    }
}

impl<N> Signed for Expr<N>
where
    Self: Num + PartialOrd + Clone + From<i32>,
    N: Signed,
{
    fn abs(&self) -> Self {
        if self.is_negative() {
            self.clone().neg()
        } else {
            self.clone()
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Self::zero()
        } else {
            self.clone() - other.clone()
        }
    }

    fn signum(&self) -> Self {
        if self.is_negative() {
            Self::one().neg()
        } else if self.is_positive() {
            Self::one()
        } else {
            Self::zero()
        }
    }

    fn is_positive(&self) -> bool {
        self.coefficient().map_or(true, Signed::is_positive)
    }

    fn is_negative(&self) -> bool {
        !self.is_zero() && !self.is_positive()
    }
}
