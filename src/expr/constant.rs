#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Const {
    /// The ratio of a circle's circumfrence to its diameter.
    Pi,

    /// The ratio of a circle's circumfrence to its radius.
    Tau,

    /// The limit of (1+1/n)^n as n approaches infinity.
    E,

    /// Euler-Mascheroni constant γ. The limiting difference between the harmonic series and the natural logarithm.
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

    /// Avogadro constant
    Na,

    /// Luminous efficacy of 540 THz radiation.
    Kcd,

    /// Reduced Planck constant.
    Hbar,

    /// Newtonian constant of gravitation.
    G,

    /// Electron mass.
    Me,

    /// Proton mass.
    Mp,
}
