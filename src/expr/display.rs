use std::ops::Neg;

use num::{traits::Inv, BigRational, One, Signed, ToPrimitive};

use crate::{
    config::Config,
    radix::{self, Radix},
};

use super::Expr;

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

impl Expr {
    /// Represents its desired position in a product; i.e., coefficients have a higher priority than variables.
    pub const fn product_priority(&self) -> u8 {
        match self {
            Expr::Num(_) => 0,
            Expr::Power(_, _) => 2,
            Expr::Log(_, _) => 1,
            Expr::Var(_) => 4,
            Expr::Const(_) => 3,
            _ => 5,
        }
    }

    /// The grouping priority of an expression represents its position in the order of operations; higher priority means further along in the order, i.e. addition has a higher priority than exponentiation.
    pub fn grouping_priority(&self) -> u8 {
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

    /// Use the grouping priority of `self` and `child` to decide wether or not to surround `child` in parens, then format it.
    pub fn display_child(
        &self,
        child: &Self,
        exact: bool,
        radix: Radix,
        config: &Config,
    ) -> String {
        if child.grouping_priority() > self.grouping_priority() || child.is_mod() {
            format!("({})", child.display(exact, radix, config))
        } else {
            child.display(exact, radix, config)
        }
    }

    /// Format this expression, but don't try to split products into a numerator and denominator.
    pub fn product_safe_format(
        &self,
        child: &Self,
        exact: bool,
        radix: Radix,
        config: &Config,
    ) -> String {
        match child {
            Self::Product(v) => {
                let str = v
                    .iter()
                    .map(|t| self.display_child(t, exact, radix, config))
                    .collect::<Vec<_>>()
                    .join("·");

                if child.grouping_priority() > self.grouping_priority() {
                    format!("({})", str)
                } else {
                    str
                }
            }
            other => self.display_child(other, exact, radix, config),
        }
    }

    /// Does this expression have a negative exponent? Will also return true for fractions with a numerator of 1.
    pub fn has_pos_exp(&self) -> bool {
        match self {
            Self::Num(n) => !n.numer().is_one(),
            other => other.exponent().map_or(true, Signed::is_positive),
        }
    }

    /// Render the sum `self`, with terms `ts`.
    pub fn display_sum(&self, ts: &[Self], exact: bool, radix: Radix, config: &Config) -> String {
        let mut s = String::new();

        let (pos, neg): (Vec<&Self>, Vec<&Self>) = ts.iter().partition(|t| t.is_positive());

        s.push_str(
            &pos.iter()
                .map(|t| self.display_child(t, exact, radix, config))
                .collect::<Vec<_>>()
                .join("+"),
        );

        for n in neg {
            s.push_str(&format!(
                "-{}",
                self.display_child(&n.clone().neg(), exact, radix, config)
            ));
        }

        s
    }

    /// Render the product `self`, with factors `fs`.
    pub fn display_product(
        &self,
        fs: &[Self],
        exact: bool,
        radix: Radix,
        config: &Config,
    ) -> String {
        let (numer_vec, denom_vec): (Vec<&Self>, Vec<&Self>) =
            fs.iter().partition(|f| f.has_pos_exp());

        let mut numer = Self::Product(numer_vec.into_iter().map(Clone::clone).collect());
        let mut denom = Self::Product(denom_vec.into_iter().map(|f| f.clone().inv()).collect());
        numer.correct();
        denom.correct();

        format!(
            "{}{}",
            self.product_safe_format(&numer, exact, radix, config),
            if denom.is_one() {
                String::new()
            } else {
                format!(
                    "/{}",
                    self.product_safe_format(&denom, exact, radix, config)
                )
            }
        )
    }

    /// Render the power expression `self`, with base `b` and exponent `e`.
    pub fn display_power(
        &self,
        b: &Self,
        e: &Self,
        exact: bool,
        radix: Radix,
        config: &Config,
    ) -> String {
        if *e == Self::from((1, 2)) {
            format!("sqrt({})", b.display(exact, radix, config))
        } else if *e == Self::from((1, 3)) {
            format!("cbrt({})", b.display(exact, radix, config))
        } else if *e == Self::from((1, 2)).neg() {
            format!("1/sqrt({})", b.display(exact, radix, config))
        } else if *e == Self::from((1, 3)).neg() {
            format!("1/cbrt({})", b.display(exact, radix, config))
        } else {
            format!(
                "{}^{}",
                self.display_child(b, exact, radix, config),
                self.display_child(e, exact, radix, config)
            )
        }
    }

    /// Render the rational expression `self`, with rational `b`.
    ///
    /// # Panics
    ///
    /// Will panic if `!exact` and `n` cannot be represented as an f64.
    #[allow(clippy::cast_precision_loss)]
    pub fn display_num(n: &BigRational, exact: bool, radix: Radix, config: &Config) -> String {
        let r = if exact {
            if radix == config.radix {
                String::new()
            } else {
                format!("{radix}#")
            }
        } else if config.radix == radix::DECIMAL {
            String::new()
        } else {
            "dec#".to_string()
        };

        if exact {
            format!("{r}{}", radix.display_bigrational(n))
        } else {
            let n = n.to_f64().unwrap();
            if n >= radix.pow(6) as f64 || n <= (*radix as f64).powi(-4) {
                format!("{r}{n:.3e}").replace('e', "ᴇ")
            } else {
                format!("{r}{n:.3}")
            }
        }
    }

    /// Render `self` to a string with the given preferences.
    pub fn display(&self, exact: bool, radix: Radix, config: &Config) -> String {
        match self {
            Self::Num(n) => Self::display_num(n, exact, radix, config),
            Self::Sum(ts) => self.display_sum(ts, exact, radix, config),
            Self::Product(fs) => self.display_product(fs, exact, radix, config),
            Self::Power(b, e) => self.display_power(b, e, exact, radix, config),
            Self::Var(s) => s.to_string(),
            Self::Const(c) => format!("{c}"),
            Self::Mod(x, y) => format!(
                "{} mod {}",
                self.display_child(x, exact, radix, config),
                self.display_child(y, exact, radix, config)
            ),
            Self::Log(b, a) => format!(
                "log({})({})",
                b.display(exact, radix, config),
                a.display(exact, radix, config)
            ),
            Self::Sin(t, m) => format!("sin({} {m})", t.display(exact, radix, config)),
            Self::Cos(t, m) => format!("cos({} {m})", t.display(exact, radix, config)),
            Self::Tan(t, m) => format!("tan({} {m})", t.display(exact, radix, config)),
            Self::Asin(t, m) => format!("(arcsin({}) {m})", t.display(exact, radix, config)),
            Self::Acos(t, m) => format!("(arccos({}) {m})", t.display(exact, radix, config)),
            Self::Atan(t, m) => format!("(arctan({}) {m})", t.display(exact, radix, config)),
        }
    }
}
