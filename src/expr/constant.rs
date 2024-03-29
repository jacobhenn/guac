use std::f64;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// Numerous common mathematical and physical constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
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

impl Const {
    /// Pretty-print this constant using unicode characters.
    pub fn display_unicode(self) -> &'static str {
        match self {
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
        }
    }

    /// Pretty-print this constant using latex macros.
    pub fn display_latex(self) -> &'static str {
        match self {
            Self::Pi => r"\pi",
            Self::Tau => r"\tau",
            Self::E => r"e",
            Self::Gamma => r"\gamma",
            Self::Vcs => r"\Delta v_{\mathrm{Cs}}",
            Self::C => r"c",
            Self::H => r"h",
            Self::Qe => r"Q_e",
            Self::K => r"k_B",
            Self::Hbar => r"\hbar",
            Self::G => r"G",
            Self::Me => r"m_e",
            Self::Mp => r"m_p",
        }
    }
}

#[allow(clippy::use_self)]
impl From<Const> for f64 {
    fn from(c: Const) -> Self {
        match c {
            Const::Pi => f64::consts::PI,
            Const::Tau => f64::consts::TAU,
            Const::E => f64::consts::E,
            Const::Gamma => 0.577_215_664_901_532_9_f64,
            Const::Vcs => 9_192_631_770_f64,
            Const::C => 299_792_458_f64,
            Const::H => 6.626_070_15e-34,
            Const::Hbar => 6.626_070_15e-34 / f64::consts::TAU,
            Const::Qe => 1.602_176_634e-19,
            Const::K => 1.380_649e-23,
            Const::G => 6.674_301_5e-11,
            Const::Me => 9.109_383_701_528e-31,
            Const::Mp => 1.672_621_923_695_1e-27,
        }
    }
}
