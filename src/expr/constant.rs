use std::{fmt::{Display, self, Formatter}, f64::consts};

/// Numerous common mathematical and physical constants.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Const {
    /// π ≈ 3.142: The ratio of a circle's circumfrence to its diameter.
    Pi,

    /// τ ≈ 6.283: The ratio of a circle's circumfrence to its radius.
    Tau,

    /// e ≈ 2.718: The limit of (1+1/n)^n as n approaches infinity.
    E,

    /// γ ≈ 0.577: Euler-Mascheroni constant. The limiting difference between the harmonic series and the natural logarithm.
    Gamma,

    /// ΔvCs ≈ 9.193ᴇ9 Hz: Hyperfine transition frequency of caesium.
    Vcs,

    /// c ≈ 2.998ᴇ8 m/s: Speed of light in vacuum.
    C,

    /// ℎ ≈ 6.626ᴇ-34 J/Hz: Planck constant.
    H,

    /// ℏ = ℎ/τ ≈ 1.054ᴇ-34 J/Hz: Reduced Planck constant.
    Hbar,

    /// e ≈ 1.602ᴇ-19 C: Elementary charge.
    Qe,

    /// k ≈ 1.380ᴇ-23 J/K: Boltzmann constant
    K,

    /// G ≈ 6.674ᴇ-11 m³·kg⁻¹·s⁻²: Newtonian constant of gravitation.
    G,

    /// mₑ ≈ 9.109ᴇ-31 kg: Electron mass.
    Me,

    /// m_p ≈ 1.673ᴇ-27 kg: Proton mass.
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

#[allow(clippy::use_self)]
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
