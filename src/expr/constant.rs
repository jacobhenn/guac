use std::{fmt::{Display, self, Formatter}, f64::consts};

/// Numerous common mathematical and physical constants.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Const {
    /// The ratio of a circle's circumfrence to its diameter.
    Pi,

    /// The ratio of a circle's circumfrence to its radius.
    Tau,

    /// The limit of (1+1/n)^n as n approaches infinity.
    E,

    /// Euler-Mascheroni constant: The limiting difference between the harmonic series and the natural logarithm.
    Gamma,

    /// Hyperfine transition frequency of caesium.
    Vcs,

    /// Speed of light.
    C,

    /// Planck constant.
    H,

    /// Elementary charge.
    Qe,

    /// Boltzmann constant
    K,

    /// Reduced Planck constant.
    Hbar,

    /// Newtonian constant of gravitation.
    G,

    /// Electron mass.
    Me,

    /// Proton mass.
    Mp,
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pi => "π",
            Self::Tau => "τ",
            Self::E => "e",
            Self::Gamma => "γ",
            Self::Vcs => "ΔvCs",
            Self::C => "c",
            Self::H => "ℎ",
            Self::Qe => "Qₑ",
            Self::K => "kB",
            Self::Hbar => "ℏ",
            Self::G => "G",
            Self::Me => "mₑ",
            Self::Mp => "mₚ",
        };
        write!(f, "{}", s)
    }
}

impl From<Const> for f64 {
    fn from(c: Const) -> Self {
        match c {
            Const::Pi => consts::PI,
            Const::Tau => consts::TAU,
            Const::E => consts::E,
            Const::Gamma => 0.5772156649015329f64,
            Const::Vcs => 9192631770f64,
            Const::C => 299792458f64,
            Const::H => 6.62607015e-34,
            Const::Hbar => 6.62607015e-34 / consts::TAU,
            Const::Qe => 1.602176634e-19,
            Const::K => 1.380649e-23,
            Const::G => 6.6743015e-11,
            Const::Me => 9.109383701528e-31,
            Const::Mp => 1.6726219236951e-27,
        }
    }
}
