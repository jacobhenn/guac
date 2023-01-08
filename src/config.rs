// clippy thinks that AngleMeasure should be named `Self` because of #[derive(Display)].
// see https://github.com/rust-lang/rust-clippy/issues/9786
#![allow(clippy::use_self)]

use crate::{
    expr::{constant::Const, Expr},
    radix::Radix,
};

use std::{fs, ops::Mul, str::FromStr};

use anyhow::{bail, Context, Result};

use derive_more::Display;

use serde::Deserialize;

use serde_with::DeserializeFromStr;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// The configuration stored in `State` which will be read from a config file in the future.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// The angle measure that will be used for trig operations.
    pub angle_measure: AngleMeasure,

    /// The "default" radix in which numbers will be inputted or displayed.
    pub radix: Radix,

    /// The number of digits to display after the radix point of approximate numbers.
    pub precision: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_measure: AngleMeasure::Radian,
            radix: Radix::DECIMAL,
            precision: 3,
        }
    }
}

impl Config {
    /// Attempt to read the configuration file from the system according to [`dirs::config_dir`].
    /// On *nix, this will look in `~/.config/guac/config.toml`. Return `Ok(None)` if the config
    // file is not present.
    pub fn get() -> Result<Option<Self>> {
        let Some(mut config_path) = dirs::config_dir() else { return Ok(None); };

        config_path.push("guac");
        config_path.push("config.toml");

        if !config_path.is_file() {
            return Ok(None);
        }

        let config_str =
            fs::read_to_string(config_path).context("config file exists, but could not be read")?;

        toml::from_str(&config_str)
            .context("config file could not be parsed")
            .map(Some)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, DeserializeFromStr)]
#[cfg_attr(test, derive(Arbitrary))]
/// A unit of angle
pub enum AngleMeasure {
    /// 1/(2π) turn.
    #[display(fmt = "rad")]
    Radian,

    /// 1 turn.
    #[display(fmt = "turns")]
    Turn,

    /// 1/400 turn.
    #[display(fmt = "grad")]
    Gradian,

    /// 1/360 turn.
    #[display(fmt = "deg")]
    Degree,

    /// 1/21600 turn.
    #[display(fmt = "min")]
    Minute,

    /// 1/1296000 turn.
    #[display(fmt = "sec")]
    Second,

    /// 1/256 turn.
    #[display(fmt = "bdeg")]
    BinaryDegree,

    /// 1/24 turn.
    #[display(fmt = "hour")]
    HourAngle,

    /// 1/32 turn.
    #[display(fmt = "point")]
    Point,

    /// 1/6400 turn.
    #[display(fmt = "mil")]
    NatoMil,
}

impl AngleMeasure {
    /// Return how many of this angle measure make up a full turn.
    #[must_use]
    pub fn full_turn<N>(self) -> Expr<N>
    where
        Expr<N>: Mul<Output = Expr<N>> + From<i32>,
    {
        match self {
            Self::Radian => Expr::from(2) * Expr::Const(Const::Pi),
            other => Expr::from(match other {
                Self::Turn => 1,
                Self::Gradian => 400,
                Self::Degree => 360,
                Self::Minute => 21600,
                Self::Second => 1_296_000,
                Self::BinaryDegree => 256,
                Self::HourAngle => 24,
                Self::Point => 32,
                Self::NatoMil => 6400,
                Self::Radian => unreachable!(),
            }),
        }
    }

    /// Return how many of this angle measure make up a full turn.
    #[must_use]
    pub const fn full_turn_f64(self) -> f64 {
        match self {
            Self::Radian => std::f64::consts::TAU,
            Self::Turn => 1.0,
            Self::Gradian => 400.0,
            Self::Degree => 360.0,
            Self::Minute => 21600.0,
            Self::Second => 1_296_000.0,
            Self::BinaryDegree => 256.0,
            Self::HourAngle => 24.0,
            Self::Point => 32.0,
            Self::NatoMil => 6400.0,
        }
    }
}

impl FromStr for AngleMeasure {
    type Err = anyhow::Error;

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
            other => bail!("inavlid angle measure '{other}'"),
        }
    }
}
