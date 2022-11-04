use crate::{
    config::Config,
    expr::Expr,
    radix::{DisplayWithContext, Radix},
};

use std::{iter::Product, ops::Neg};

use num::{traits::Inv, BigRational, One, Signed, Zero};

/// **Input must be a number which has been correctly `to_string`ed.** Returns the input in e-notation. Since it takes a pre-formatted string, this works regardless of base.
// pub fn make_e_notation(mut s: String) -> String {
//     if s.contains('.') {
//         let mut ns = s.split('.');
//         let int = ns.next();
//         let decimal = ns.next();
//         todo!()
//     } else {
//         let exponent = s.len() - 1;
//         s.truncate(4);
//         for _ in 0..(4usize.saturating_sub(s.len())) {
//             s.push('0');
//         }

//         s.insert(1, '.');
//         format!("{s}ᴇ{exponent}")
//     }
// }

impl<N> Expr<N> where N: Signed {}

/// **Expression** types for which you can tell the sign of their exponent, sometimes in a smart
/// way. Ideally, this should be blanket implemented for all `Expr<T> where T: Signed` paired
/// with a specialization for `Expr<BigRational>`, but until specialization, this will just be
/// manually implemented for all needed `Expr<N>`s.
pub trait HasPosExp {
    /// Does this expression have a positive exponent? Will also return false for fractions with a
    /// numerator of 1.
    fn has_pos_exp(&self) -> bool;
}

impl HasPosExp for Expr<BigRational> {
    fn has_pos_exp(&self) -> bool {
        match self {
            Self::Num(n) => !n.numer().is_one(),
            other => other.exponent().map_or(true, Self::is_positive),
        }
    }
}

impl HasPosExp for Expr<f64> {
    fn has_pos_exp(&self) -> bool {
        self.exponent().map_or(true, Self::is_positive)
    }
}

impl<N> Expr<N>
where
    Self: HasPosExp + Clone + Inv<Output = Self> + One + Signed + From<(i32, i32)>,
    N: Zero + One + Clone + for<'a> Product<&'a N> + PartialEq + Signed + DisplayWithContext,
{
    /// Render the sum `self`, with terms `ts`.
    pub fn display_sum(&self, ts: &[Self], radix: Radix, config: &Config) -> String {
        let mut s = String::new();

        let (pos, neg): (Vec<&Self>, Vec<&Self>) = ts.iter().partition(|t| t.is_positive());

        s.push_str(
            &pos.iter()
                .map(|t| self.display_child(t, radix, config))
                .collect::<Vec<_>>()
                .join("+"),
        );

        for n in neg {
            s.push_str(&format!(
                "-{}",
                self.display_child(&n.clone().neg(), radix, config)
            ));
        }

        s
    }

    /// Use the grouping priority of `self` and `child` to decide wether or not to surround `child` in parens, then format it.
    pub fn display_child(&self, child: &Self, radix: Radix, config: &Config) -> String {
        if child.grouping_priority() > self.grouping_priority() || child.is_mod() {
            format!("({})", child.display(radix, config))
        } else {
            child.display(radix, config)
        }
    }

    /// Format this expression, but don't try to split products into a numerator and denominator.
    pub fn product_safe_format(&self, child: &Self, radix: Radix, config: &Config) -> String {
        match child {
            Self::Product(v) => {
                let str = v
                    .iter()
                    .map(|t| self.display_child(t, radix, config))
                    .collect::<Vec<_>>()
                    .join("·");

                if child.grouping_priority() > self.grouping_priority() {
                    format!("({})", str)
                } else {
                    str
                }
            }
            other => self.display_child(other, radix, config),
        }
    }

    /// Render the product `self`, with factors `fs`.
    pub fn display_product(&self, fs: &[Self], radix: Radix, config: &Config) -> String {
        let (numer_vec, denom_vec): (Vec<&Self>, Vec<&Self>) =
            fs.iter().partition(|&f| f.has_pos_exp());

        let mut numer = Self::Product(numer_vec.into_iter().map(Clone::clone).collect());
        let mut denom = Self::Product(denom_vec.into_iter().map(|f| f.clone().inv()).collect());
        numer.correct();
        denom.correct();

        format!(
            "{}{}",
            self.product_safe_format(&numer, radix, config),
            if denom.is_one() {
                String::new()
            } else {
                format!("/{}", self.product_safe_format(&denom, radix, config))
            }
        )
    }

    /// Render the power expression `self`, with base `b` and exponent `e`.
    pub fn display_power(&self, b: &Self, e: &Self, radix: Radix, config: &Config) -> String {
        if *e == Self::from((1, 2)) {
            format!("sqrt({})", b.display(radix, config))
        } else if *e == Self::from((1, 3)) {
            format!("cbrt({})", b.display(radix, config))
        } else if *e == Self::from((1, 2)).neg() {
            format!("1/sqrt({})", b.display(radix, config))
        } else if *e == Self::from((1, 3)).neg() {
            format!("1/cbrt({})", b.display(radix, config))
        } else {
            format!(
                "{}^{}",
                self.display_child(b, radix, config),
                self.display_child(e, radix, config)
            )
        }
    }

    /// Render `self` to a string with the given preferences.
    pub fn display(&self, radix: Radix, config: &Config) -> String {
        match self {
            Self::Num(n) => n.display_in(radix, config),
            Self::Sum(ts) => self.display_sum(ts, radix, config),
            Self::Product(fs) => self.display_product(fs, radix, config),
            Self::Power(b, e) => self.display_power(b, e, radix, config),
            Self::Var(s) => s.to_string(),
            Self::Const(c) => format!("{c}"),
            Self::Mod(x, y) => format!(
                "{} mod {}",
                self.display_child(x, radix, config),
                self.display_child(y, radix, config)
            ),
            Self::Log(b, a) => format!(
                "log({})({})",
                b.display(radix, config),
                a.display(radix, config)
            ),
            Self::Sin(t, m) => format!("sin({} {m})", t.display(radix, config)),
            Self::Cos(t, m) => format!("cos({} {m})", t.display(radix, config)),
            Self::Tan(t, m) => format!("tan({} {m})", t.display(radix, config)),
            Self::Asin(t, m) => format!("(arcsin({}) {m})", t.display(radix, config)),
            Self::Acos(t, m) => format!("(arccos({}) {m})", t.display(radix, config)),
            Self::Atan(t, m) => format!("(arctan({}) {m})", t.display(radix, config)),
        }
    }
}

impl<N> Expr<N> {
    /// The grouping priority of an expression represents its position in the order of operations;
    /// higher priority means further along in the order, i.e. addition has a higher priority than exponentiation.
    pub fn grouping_priority(&self) -> u8
    where
        N: Signed,
    {
        match self {
            Self::Num(n) => {
                if n.is_negative() {
                    4
                } else {
                    0
                }
            }
            Self::Power(..) => 1,
            Self::Product(..) => 2,
            Self::Sum(..) => 3,
            _ => 0,
        }
    }

    /// Represents its desired position in a product; i.e., coefficients have a higher priority than
    /// variables.
    pub const fn product_priority(&self) -> u8 {
        match self {
            Self::Num(_) => 0,
            Self::Power(_, _) => 2,
            Self::Log(_, _) => 1,
            Self::Var(_) => 4,
            Self::Const(_) => 3,
            _ => 5,
        }
    }
}
