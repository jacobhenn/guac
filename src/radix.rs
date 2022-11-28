use crate::{config::Config, expr::Expr};

use std::{fmt::Display, num::NonZeroUsize, str::FromStr};

use num::{bigint::Sign, BigInt, BigRational, One, Signed};

#[cfg(test)]
use proptest::prelude::Strategy;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// A list of Misalian radix abbreviations. The `b-2`th element contains the abbreviation for
/// base `b`.
pub const ABBVS: [&str; 63] = [
    "bin", "tri", "qua", "qui", "sex", "sep", "oct", "non", "dec", "ele", "doz", "bak", "bis",
    "trq", "hex", "sub", "trs", "unt", "vig", "tis", "bie", "unb", "tet", "pen", "bik", "trn",
    "ter", "utt", "pet", "unp", "ttr", "trl", "bib", "pnt", "nif", "unn", "bit", "trk", "pec",
    "upn", "hes", "unh", "tel", "pnn", "bnb", "ubn", "hec", "hep", "peg", "trb", "tek", "unr",
    "hen", "pel", "het", "tin", "bnt", "ubt", "heg", "unx", "bip", "hpt", "occ",
];

/// The full list of `guac`'s octoctal digits.
pub const DIGITS: [char; 64] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', '!', '@',
];

/// A radix. This will always contain something within the range 2..=64.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Radix(
    #[cfg_attr(
        test,
        proptest(
            strategy = "(2..=64usize).prop_map(|n| unsafe { NonZeroUsize::new_unchecked(n) })"
        )
    )]
    NonZeroUsize,
);

impl Radix {
    /// bin / 2: base dec#2
    pub const BINARY: Self = unsafe { Self::new_unchecked(2) };
    /// tri / 3: base dec#3
    pub const TRINARY: Self = unsafe { Self::new_unchecked(3) };
    /// qua / 4: base dec#4
    pub const QUATERNARY: Self = unsafe { Self::new_unchecked(4) };
    /// sex / 6: base dec#6
    pub const SEXIMAL: Self = unsafe { Self::new_unchecked(6) };
    /// oct / 8: base dec#8
    pub const OCTAL: Self = unsafe { Self::new_unchecked(8) };
    /// dec / a: base dec#10
    pub const DECIMAL: Self = unsafe { Self::new_unchecked(10) };
    /// doz / c: base dec#12
    pub const DOZENAL: Self = unsafe { Self::new_unchecked(12) };
    /// hex / g: base dec#16
    pub const HEX: Self = unsafe { Self::new_unchecked(16) };
    /// ttr / w: base dec#32
    pub const TETROCTAL: Self = unsafe { Self::new_unchecked(32) };
    /// nif / A: base dec#36
    pub const NIFTIMAL: Self = unsafe { Self::new_unchecked(36) };
    /// heg / Y: base dec#60
    pub const HEXAGESIMAL: Self = unsafe { Self::new_unchecked(60) };
    /// occ: base dec#64
    pub const OCTOCTAL: Self = unsafe { Self::new_unchecked(64) };

    /// Creates a radix from an integer without bounds checks.
    ///
    /// # Safety
    ///
    /// The given integer must be in the inclusive range `2..=64`, or this will result in undefined
    /// behavior.
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(n: usize) -> Self {
        unsafe { Self(NonZeroUsize::new_unchecked(n)) }
    }

    /// Create a valid radix from an integer. Returns `None` if `n` is outside the range `2..=64`.
    #[must_use]
    pub const fn new(n: usize) -> Option<Self> {
        if n > 1 && n < 65 {
            Some(unsafe { Self::new_unchecked(n) })
        } else {
            None
        }
    }

    /// Return the inner value as a primitive usize.
    #[inline]
    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }

    /// Get this radix's Misalian abbreviation from `ABBVS`.
    #[must_use]
    pub const fn abbv(&self) -> &'static str {
        ABBVS[self.get() - 2]
    }

    /// Get this radix's octoctal single-char name from `DIGITS`.
    #[must_use]
    pub fn char(&self) -> Option<&char> {
        DIGITS.get(self.get())
    }

    /// Attempt to parse a digit into an integer in this radix.
    #[must_use]
    pub fn parse_digit(&self, digit: &char) -> Option<u8> {
        let unchecked_digit: usize = DIGITS.iter().position(|c| c == digit)?;
        if unchecked_digit >= self.get() {
            None
        } else {
            Some(unchecked_digit as u8)
        }
    }

    /// Is `digit` one of the digits which can constitute a valid number in this radix?
    #[must_use]
    pub fn contains_digit(&self, digit: &char) -> bool {
        DIGITS[0..self.get()].iter().any(|c| c == digit)
    }

    /// Parse a string into a `BigInt` under this radix.
    #[must_use]
    pub fn parse_bigint(&self, s: &str) -> Option<BigInt> {
        if s.is_empty() {
            return None;
        }

        let negative = s.starts_with('-');
        let mut chars = s.chars();
        if negative {
            chars.next();
        }

        let buf: Option<Vec<u8>> = chars
            .map(|c| self.parse_digit(&c))
            .collect();

        BigInt::from_radix_be(
            if negative { Sign::Minus } else { Sign::Plus },
            &buf?,
            self.get() as u32,
        )
    }
}

impl From<Radix> for Expr<BigRational> {
    fn from(radix: Radix) -> Self {
        Self::Num(BigRational::from(BigInt::from(radix.get())))
    }
}

impl From<Radix> for Expr<f64> {
    fn from(radix: Radix) -> Self {
        Self::Num(radix.get() as f64)
    }
}

/// An error returned when a radix could not be parsed from a string.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("expected a Misalian radix abbreviation or single digit")]
pub struct ParseRadixErr;

impl FromStr for Radix {
    type Err = ParseRadixErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 3 {
            Ok(ABBVS
                .iter()
                .position(|&c| c == s)
                .map(|i| Self::new(i + 2).unwrap())
                .ok_or(ParseRadixErr)?)
        } else if s.len() == 1 {
            let c = s.chars().next().unwrap();
            Ok(DIGITS
                .iter()
                .position(|&d| d == c)
                .map(|n| Self::new(n).unwrap())
                .ok_or(ParseRadixErr)?)
        } else {
            Err(ParseRadixErr)
        }
    }
}

impl Display for Radix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ABBVS[self.get() - 2])
    }
}

/// Types which can be displayed given the surrounding context of a radix and a configuration.
/// If we had `with` clauses, this could probably be replaced by
/// `fmt::Display with(Radix, &Config)`
// TODO: make these write to a buffer instead of returning new strings
pub trait DisplayWithContext {
    /// Returns what prefix should be put in front of this number when displaying in the given
    /// context. For example, `prefix(Radix::DECIMAL, config)` will return an empty string if
    /// the current global radix is decimal, and "dec#" if it is not.
    #[must_use]
    fn prefix(radix: Radix, config: &Config) -> String {
        if config.radix == radix {
            String::new()
        } else {
            format!("{radix}#")
        }
    }

    /// Displays the number `self` in the given radix and context, **without** any radix-specific
    /// prefix. For example,
    /// `<BigInt as DisplayWithContext>::display_impl(BigInt::from(5), Radix::BINARY, config)`
    /// returns only "110", whether or not the current global radix is binary.
    fn display_impl(&self, radix: Radix, config: &Config) -> String;

    /// Completely displays the number `self` in the given radix and context, including a radix
    /// prefix if the given radix does not match the current global radix.
    fn display_in(&self, radix: Radix, config: &Config) -> String {
        format!(
            "{}{}",
            Self::prefix(radix, config),
            self.display_impl(radix, config)
        )
    }
}

impl DisplayWithContext for BigInt {
    fn display_impl(&self, radix: Radix, _: &Config) -> String {
        let mut s = String::new();
        let (sign, buf) = self.to_radix_be(radix.get() as u32);
        if sign == Sign::Minus {
            s.push('-');
        }

        for digit in buf {
            s.push(DIGITS[digit as usize]);
        }

        s
    }
}

impl DisplayWithContext for BigRational {
    fn display_impl(&self, radix: Radix, cfg: &Config) -> String {
        if self.is_negative() {
            format!("-{}", self.abs().display_impl(radix, cfg))
        } else {
            let mut s = String::new();
            let numer = self.numer();
            let denom = self.denom();
            s.push_str(&numer.display_impl(radix, cfg));
            if !denom.is_one() {
                s.push('/');
                s.push_str(&denom.display_impl(radix, cfg));
            }

            s
        }
    }
}

// TODO: make this work in all radices
// TODO: add configurable display precision
impl DisplayWithContext for f64 {
    fn prefix(_: Radix, config: &Config) -> String {
        if config.radix == Radix::DECIMAL {
            String::new()
        } else {
            format!("{}#", Radix::DECIMAL)
        }
    }

    fn display_impl(&self, _: Radix, _: &Config) -> String {
        if *self >= 1e6 || *self <= 1e-4 {
            format!("{self:.3e}")
        } else {
            format!("{self:.3}")
        }
    }
}
