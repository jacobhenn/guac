use std::ops::{Add, Sub, Mul, Div};
use std::{str, fmt};

#[derive(Clone, Copy)]
pub enum Real {
    Integer(i32),
    Rational(i32, i32),
}

impl fmt::Display for Real {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{}", i),
            Self::Rational(n, d) => write!(f, "{}/{}", n, d),
        }
    }
}

impl str::FromStr for Real {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Integer(s.parse()?))
    }
}

impl Add for Real {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Integer(i), Self::Integer(o)) => Self::Integer(i + o),
            (r, Self::Rational(n, d)) =>
                (Self::Integer(n) + (r * Self::Integer(d))) / Self::Integer(d),
            (Self::Rational(n, d), r) =>
                (Self::Integer(n) + (r * Self::Integer(d))) / Self::Integer(d),
        }
    }
}

impl Sub for Real {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Self::Integer(i), Self::Integer(o)) => Self::Integer(i - o),
            (r, Self::Rational(n, d)) =>
                ((r * Self::Integer(d)) - Self::Integer(n)) / Self::Integer(d),
            (Self::Rational(n, d), r) =>
                (Self::Integer(n) - (r * Self::Integer(d))) / Self::Integer(d),
        }
    }
}

impl Mul for Real {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Self::Integer(i), Self::Integer(o)) => Self::Integer(i * o),
            (Self::Integer(i), Self::Rational(n, d)) => Self::Integer(n * i) / Self::Integer(d),
            (Self::Rational(n, d), Self::Integer(i)) => Self::Integer(n * i) / Self::Integer(d),
            (Self::Rational(n, d), Self::Rational(m, e)) =>
                Self::Integer(n * m) / Self::Integer(d * e),
        }
    }
}

impl Div for Real {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Self::Integer(i), Self::Integer(o)) => {
                if i % o == 0 {
                    Self::Integer(i / o)
                } else {
                    let gcd = gcd(i, o);
                    Self::Rational(i / gcd, o / gcd)
                }
            }
            (r, Self::Rational(n, d)) => r * (Self::Integer(d) / Self::Integer(n)),
            (s @ Self::Rational(..), o @ Self::Integer(_)) => s * (Self::Integer(1) / o),
        }
    }
}

// Euclid's two-thousand-year-old algorithm for finding the greatest common divisor
fn gcd(x: i32, y: i32) -> i32 {
    let mut x = x;
    let mut y = y;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}
