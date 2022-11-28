use crate::{
    config::{AngleMeasure, Config},
    expr::{Const, Expr},
    radix::{DisplayWithContext, Radix},
};

use std::{fmt, ops::Neg};

use num::{traits::Inv, BigRational, One, Signed};

/// Display `Expr`s in latex notation.
// pub mod latex;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Precedence {
    Zero,
    Power,
    Product,
    Sum,
    Negative,
}

/// A buffer with associated context to which expressions can be written. Everything that
/// implements this should have a field that implements `fmt::Write`.
pub trait ExprFormatter<N>: Sized
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
{
    #![allow(missing_docs)]

    /// The type returned in event of a formatting error.
    type Error: From<fmt::Error>;

    /// Get a mutable reference to the internal buffer.
    fn get_buf(&mut self) -> &mut dyn fmt::Write;

    /// Format the given inner item in parentheses.
    fn fmt_in_parens(&mut self, inner: impl Formattable<N, Self>) -> Result<(), Self::Error>;

    fn fmt_fn_call(
        &mut self,
        name: impl Formattable<N, Self>,
        inner: impl Formattable<N, Self>,
    ) -> Result<(), Self::Error>;

    /// Format the given inner item to the buffer; if its precedence is higher than the given
    /// precedence of the outer expression, format it in parentheses.
    fn fmt_child(
        &mut self,
        parent_precedence: Precedence,
        child: &Expr<N>,
    ) -> Result<(), Self::Error> {
        if parent_precedence < child.precedence() {
            self.fmt_in_parens(child)
        } else {
            self.fmt(child)
        }
    }

    /// Format the given expression to the buffer.
    fn fmt(&mut self, expr: &Expr<N>) -> Result<(), Self::Error> {
        match expr {
            Expr::Num(n) => self.fmt_num(n),
            Expr::Sum(ts) => self.fmt_sum(ts),
            Expr::Product(fs) => self.fmt_product(fs),
            Expr::Power(b, e) => self.fmt_power(b, e),
            Expr::Log(b, a) => self.fmt_log(b, a),
            Expr::Var(s) => self.fmt_var(s),
            Expr::Const(c) => self.fmt_const(*c),
            Expr::Mod(x, y) => self.fmt_mod(x, y),
            Expr::Sin(x, m) => self.fmt_sin(x, *m),
            Expr::Cos(x, m) => self.fmt_cos(x, *m),
            Expr::Tan(x, m) => self.fmt_tan(x, *m),
            Expr::Asin(x, m) => self.fmt_asin(x, *m),
            Expr::Acos(x, m) => self.fmt_acos(x, *m),
            Expr::Atan(x, m) => self.fmt_atan(x, *m),
        }
    }

    /// Format a single number to the buffer.
    fn fmt_num(&mut self, num: &N) -> Result<(), Self::Error>;

    /// Format a sum of terms to the buffer.
    fn fmt_sum(&mut self, terms: &[Expr<N>]) -> Result<(), Self::Error> {
        let mut terms_iter = terms.iter().filter(|t| t.is_positive()).peekable();
        while let Some(term) = terms_iter.next() {
            self.fmt_child(Precedence::Sum, term)?;
            if terms_iter.peek().is_some() {
                self.get_buf().write_char('+')?;
            }
        }

        for term in terms
            .iter()
            .filter(|t| t.is_negative())
            .map(|t| t.clone().neg())
        {
            self.get_buf().write_char('-')?;
            self.fmt_child(Precedence::Sum, &term)?;
        }

        Ok(())
    }

    /// Write the separating string that should go in between factors of a product ('·' for the
    /// [default formatter](DefaultFormatter), "\cdot" for the [latex formatter](LatexFormatter)).
    fn write_product_separator(&mut self) -> Result<(), Self::Error>;

    /// Format the numerator or denominator of a fraction to the buffer.
    fn fmt_frac_component(
        &mut self,
        factors: impl Iterator<Item = impl Formattable<N, Self>>,
    ) -> Result<(), Self::Error> {
        let mut factors = factors.peekable();
        while let Some(factor) = factors.next() {
            factor.fmt_to(self)?;
            if factors.peek().is_some() {
                self.write_product_separator()?;
            }
        }

        Ok(())
    }

    /// Format a fraction to the buffer.
    fn fmt_frac(
        &mut self,
        numer: impl Iterator<Item = impl Formattable<N, Self>>,
        denom: impl Iterator<Item = impl Formattable<N, Self>>,
    ) -> Result<(), Self::Error>;

    /// Format a product of factors to the buffer as numerator and denominator.
    fn fmt_product(&mut self, factors: &[Expr<N>]) -> Result<(), Self::Error> {
        let numer = factors.iter().filter(|f| f.has_pos_exp());
        let denom = factors
            .iter()
            .filter(|f| !f.has_pos_exp())
            .map(|f| f.clone().inv());

        if factors.iter().all(Expr::has_pos_exp) {
            self.fmt_frac_component(numer)
        } else {
            self.fmt_frac(numer, denom)
        }
    }

    fn fmt_power(&mut self, base: &Expr<N>, exp: &Expr<N>) -> Result<(), Self::Error>;
    fn fmt_log(&mut self, base: &Expr<N>, arg: &Expr<N>) -> Result<(), Self::Error>;
    fn fmt_var(&mut self, var: &str) -> Result<(), Self::Error>;
    fn fmt_const(&mut self, cnst: Const) -> Result<(), Self::Error>;
    fn fmt_mod(&mut self, lhs: &Expr<N>, rhs: &Expr<N>) -> Result<(), Self::Error>;

    fn fmt_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        units: AngleMeasure,
    ) -> Result<(), Self::Error>;

    fn fmt_sin(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_trig("sin", arg, units)
    }

    fn fmt_cos(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_trig("cos", arg, units)
    }

    fn fmt_tan(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_trig("tan", arg, units)
    }

    fn fmt_inv_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        units: AngleMeasure,
    ) -> Result<(), Self::Error>;

    fn fmt_asin(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_inv_trig("asin", arg, units)
    }

    fn fmt_acos(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_inv_trig("acos", arg, units)
    }

    fn fmt_atan(&mut self, arg: &Expr<N>, units: AngleMeasure) -> Result<(), Self::Error> {
        self.fmt_inv_trig("atan", arg, units)
    }
}

// TODO: see if there's a better way to do this. it seems like there should be
/// Something which can be formatted to an `ExprFormatter`.
pub trait Formattable<N, F>
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    F: ExprFormatter<N>,
{
    /// Format the given value to the given formatter.
    fn fmt_to(&self, f: &mut F) -> Result<(), F::Error>;
}

#[allow(clippy::use_self)]
impl<N, F> Formattable<N, F> for Expr<N>
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    F: ExprFormatter<N>,
{
    fn fmt_to(&self, f: &mut F) -> Result<(), F::Error> {
        f.fmt(self)
    }
}

impl<N, F> Formattable<N, F> for &Expr<N>
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    F: ExprFormatter<N>,
{
    fn fmt_to(&self, f: &mut F) -> Result<(), F::Error> {
        f.fmt(self)
    }
}

impl<N, F> Formattable<N, F> for &str
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    F: ExprFormatter<N>,
{
    fn fmt_to(&self, f: &mut F) -> Result<(), F::Error> {
        f.get_buf().write_str(self).map_err(Into::into)
    }
}

impl<N, F, G> Formattable<N, F> for G
where
    N: Signed,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    F: ExprFormatter<N>,
    G: Fn(&mut F) -> Result<(), F::Error>,
{
    fn fmt_to(&self, f: &mut F) -> Result<(), F::Error> {
        self(f)
    }
}

/// The [formatter](ExprFormatter) for writing expressions to the stack under normal operation.
pub struct DefaultFormatter<'a> {
    config: &'a Config,
    radix: Radix,
    buf: &'a mut (dyn fmt::Write + 'a),
}

impl<'a> DefaultFormatter<'a> {
    /// Create a new [`DefaultFormatter`] which writes into `buf`.
    pub fn new(config: &'a Config, radix: Radix, buf: &'a mut (dyn fmt::Write + 'a)) -> Self {
        Self { config, radix, buf }
    }
}

impl<'a, N> ExprFormatter<N> for DefaultFormatter<'a>
where
    N: Signed + DisplayWithContext,
    Expr<N>:
        Signed + HasPosExp + Clone + Inv<Output = Expr<N>> + From<(i32, i32)> + PartialEq<Expr<N>>,
{
    type Error = fmt::Error;

    #[inline]
    fn get_buf(&mut self) -> &mut dyn fmt::Write {
        self.buf
    }

    fn fmt_in_parens(&mut self, inner: impl Formattable<N, Self>) -> Result<(), Self::Error>
    where
        Expr<N>: Signed,
        N: DisplayWithContext,
    {
        self.buf.write_char('(')?; // :)
        inner.fmt_to(self)?;
        self.buf.write_char(')')?;
        Ok(())
    }

    fn fmt_fn_call(
        &mut self,
        name: impl Formattable<N, Self>,
        inner: impl Formattable<N, Self>,
    ) -> Result<(), Self::Error> {
        name.fmt_to(self)?;
        self.buf.write_char('(')?; // )
        inner.fmt_to(self)?;
        self.buf.write_char(')')?;
        Ok(())
    }

    fn fmt_num(&mut self, num: &N) -> Result<(), Self::Error>
    where
        N: DisplayWithContext,
    {
        write!(self.buf, "{}", num.display_in(self.radix, self.config))
    }

    fn write_product_separator(&mut self) -> Result<(), Self::Error> {
        self.buf.write_char('·')
    }

    fn fmt_frac(
        &mut self,
        numer: impl Iterator<Item = impl Formattable<N, Self>>,
        denom: impl Iterator<Item = impl Formattable<N, Self>>,
    ) -> Result<(), Self::Error> {
        self.fmt_frac_component(numer)?;
        self.buf.write_char('/')?;
        self.fmt_frac_component(denom)?;
        Ok(())
    }

    fn fmt_power(&mut self, base: &Expr<N>, exp: &Expr<N>) -> Result<(), Self::Error> {
        if *exp == Expr::from((1, 2)) {
            self.buf.write_str("sqrt")?;
            self.fmt_in_parens(base)?;
        } else if *exp == Expr::from((1, 3)) {
            self.buf.write_str("cbrt")?;
            self.fmt_in_parens(base)?;
        } else {
            self.fmt_child(Precedence::Power, base)?;
            self.buf.write_char('^')?;
            self.fmt_child(Precedence::Power, exp)?;
        }

        Ok(())
    }

    fn fmt_log(&mut self, base: &Expr<N>, arg: &Expr<N>) -> Result<(), Self::Error> {
        self.buf.write_str("log")?;
        self.fmt_in_parens(base)?;
        self.fmt_in_parens(arg)?;
        Ok(())
    }

    fn fmt_var(&mut self, var: &str) -> Result<(), Self::Error> {
        self.buf.write_str(var)
    }

    fn fmt_const(&mut self, cnst: Const) -> Result<(), Self::Error> {
        write!(self.buf, "{cnst}")
    }

    fn fmt_mod(&mut self, lhs: &Expr<N>, rhs: &Expr<N>) -> Result<(), Self::Error> {
        self.fmt_child(Precedence::Product, lhs)?;
        self.buf.write_char('%')?;
        self.fmt_child(Precedence::Product, rhs)?;
        Ok(())
    }

    fn fmt_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        units: AngleMeasure,
    ) -> Result<(), Self::Error> {
        func.fmt_to(self)?;
        self.fmt_in_parens(|this: &mut Self| {
            this.fmt(arg)?;
            write!(this.get_buf(), " {units}")?;
            Ok(())
        })?;

        Ok(())
    }

    fn fmt_inv_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        units: AngleMeasure,
    ) -> Result<(), Self::Error> {
        self.fmt_in_parens(|this: &mut Self| {
            func.fmt_to(this)?;
            this.fmt_in_parens(arg)?;
            write!(this.get_buf(), " {units}")?;
            Ok(())
        })?;

        Ok(())
    }
}

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

impl<N> Expr<N> {
    /// Returns the [`Precedence`] of this expression (its position in the order of operations).
    pub fn precedence(&self) -> Precedence
    where
        N: Signed,
    {
        match self {
            Self::Num(n) => {
                if n.is_negative() {
                    Precedence::Negative
                } else {
                    Precedence::Zero
                }
            }
            Self::Power(..) => Precedence::Power,
            Self::Product(..) => Precedence::Product,
            Self::Sum(..) => Precedence::Sum,
            _ => Precedence::Zero,
        }
    }

    /// Represents its desired position in a product; i.e., coefficients have a higher priority
    /// than variables.
    // TODO: does this really need to be covered by the blanket `N: Signed` bound on this impl?
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

    /// Displays the given expression in the given radix with the given configuration using the
    /// [default formatter](DefaultFormatter)
    ///
    /// # Panics
    ///
    /// This function could theoretically panic if `<String as fmt::Write>::write_str` panics. As
    /// of the 1.65.0 standard library, this is strictly impossible.
    pub fn display(&self, radix: Radix, config: &Config) -> String
    where
        N: Signed,
        Self: HasPosExp + Inv<Output = Self> + Clone + Signed,
        for<'a> DefaultFormatter<'a>: ExprFormatter<N>,
        for<'a> <DefaultFormatter<'a> as ExprFormatter<N>>::Error: fmt::Debug,
    {
        let mut s = String::new();
        let mut formatter = DefaultFormatter::new(config, radix, &mut s);
        formatter.fmt(self).unwrap();
        s
    }
}
