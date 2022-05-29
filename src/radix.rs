use std::{str::FromStr, fmt::Display};

use num::{BigInt, bigint::Sign, BigRational, One, Signed};

use crate::expr::Expr;

/// A list of Misalian radix abbreviations. The `b-2`th element contains the abbreviation for base `b`.
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

/// A radix. Panics will happen if this contains anything outside of 2..=64.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Radix(usize);

/// bin / 2: base dec#2
pub const BINARY: Radix = Radix(2);
/// tri / 3: base dec#3
pub const TRINARY: Radix = Radix(3);
/// qua / 4: base dec#4
pub const QUATERNARY: Radix = Radix(4);
/// sex / 6: base dec#6
pub const SEXIMAL: Radix = Radix(6);
/// oct / 8: base dec#8
pub const OCTAL: Radix = Radix(8);
/// dec / a: base dec#10
pub const DECIMAL: Radix = Radix(10);
/// doz / c: base dec#12
pub const DOZENAL: Radix = Radix(12);
/// hex / g: base dec#16
pub const HEX: Radix = Radix(16);
/// ttr / w: base dec#32
pub const TETROCTAL: Radix = Radix(32);
/// nif / A: base dec#36
pub const NIFTIMAL: Radix = Radix(36);
/// heg / Y: base dec#60
pub const HEXAGESIMAL: Radix = Radix(60);
/// occ: base dec#64
pub const OCTOCTAL: Radix = Radix(64);

impl Radix {
    /// Create a valid Radix from an integer.
    pub fn from_int<I>(n: I) -> Option<Self> where I: Into<usize> {
        let n = n.into();
        if (2..=64).contains(&n) {
            Some(Self(n))
        } else {
            None
        }
    }

    /// Get this radix's Misalian abbreviation from `ABBVS`.
    pub const fn abbv(&self) -> &'static str {
        ABBVS[self.0 - 2]
    }

    /// Get this radix's octoctal single-char name from `DIGITS`.
    pub fn char(&self) -> Option<&char> {
        DIGITS.get(self.0)
    }

    /// Attempt to parse a digit into an integer in this radix.
    pub fn parse_digit<T>(&self, digit: &char) -> Option<T> where T: TryFrom<usize> {
        let unchecked_digit: usize = DIGITS.iter().position(|c| c == digit)?;
        if unchecked_digit >= self.0 {
            None
        } else {
            Some(unchecked_digit.try_into().ok()?)
        }
    }

    /// Is `digit` one of the digits which can constitute a valid number in this radix?
    pub fn contains_digit(&self, digit: &char) -> bool {
        DIGITS[0..self.0].iter().any(|c| c == digit)
    }

    /// Parse a string into a `BigInt` under this radix.
    pub fn parse_bigint(&self, s: &str) -> Option<BigInt> {
        if s.is_empty() {
            return None;
        }

        let buf: Option<Vec<u8>> = s.chars().map(|c| self.parse_digit::<u8>(&c)).collect();
        BigInt::from_radix_be(Sign::Plus, &buf?, self.0 as u32)
    }

    /// Turn a `BigInt` into a string under this radix.
    pub fn display_bigint(&self, i: &BigInt) -> String {
        let mut s = String::new();
        let (sign, buf) = i.to_radix_be(self.0 as u32);
        if sign == Sign::Minus {
            s.push('-');
        }

        for digit in buf {
            s.push(DIGITS[digit as usize]);
        }

        s
    }

    /// Display a `BigRational` as a `String` under this radix.
    pub fn display_bigrational(&self, i: &BigRational) -> String {
        let mut s = String::new();
        if i.is_negative() {
            s.push('-');
            s.push_str(&self.display_bigrational(&i.abs()));
        } else {
            let numer = i.numer();
            let denom = i.denom();
            s.push_str(&self.display_bigint(numer));
            if !denom.is_one() {
                s.push('/');
                s.push_str(&self.display_bigint(denom));
            }
        }

        s
    }
}

impl From<Radix> for Expr {
    fn from(r: Radix) -> Self {
        Self::from_int(r.0 as i128)
    }
}

impl FromStr for Radix {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 3 {
            Ok(Self(ABBVS.iter().position(|c| c == &s).map(|i| i + 2).ok_or(())?))
        } else if s.len() == 1 {
            let c = s.chars().next().unwrap();
            Ok(Self(DIGITS.iter().position(|d| d == &c).ok_or(())?))
        } else {
            Err(())
        }
    }
}

impl Display for Radix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ABBVS[self.0 - 2])
    }
}
