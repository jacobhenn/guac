use crate::{
    config::{AngleMeasure, Config},
    expr::{
        constant::Const,
        display::{ExprFormatter, Formattable, HasPosExp},
        Expr,
    },
    radix::{DisplayWithContext, Radix},
};

use std::fmt;

use derive_more::Display;

use num::{traits::Inv, Signed};

/// An error encountered when formatting an expression in latex.
#[derive(Display, Debug, Clone)]
pub enum Error {
    /// The expression contained a non-ascii variable name. Latex does not like non-ascii text.
    #[display(fmt = "non-ascii")]
    NonAsciiVariable,

    /// The expression contained a variable whose name contained a backslash. These and others will
    /// be escaped in the future.
    #[display(fmt = "'\\' in var")]
    BackslashInVar,

    /// The format failed because of an internal i/o error.
    #[display(fmt = "internal error")]
    FmtError(fmt::Error),
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self::FmtError(err)
    }
}

/// The formatter used to display an expression in latex.
pub struct Formatter<'a> {
    config: &'a Config,
    radix: Radix,
    buf: &'a mut (dyn fmt::Write + 'a),
}

impl<'a> Formatter<'a> {
    /// Create a new [`Formatter`] which writes into `buf`.
    pub fn new(config: &'a Config, radix: Radix, buf: &'a mut (dyn fmt::Write + 'a)) -> Self {
        Self { config, radix, buf }
    }

    fn fmt_latex_call<N>(
        &mut self,
        name: impl Formattable<N, Self>,
        inner: impl Formattable<N, Self>,
    ) -> Result<(), Error>
    where
        N: Signed + DisplayWithContext,
        Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
    {
        self.buf.write_char('\\')?;
        name.fmt_to(self)?;
        self.buf.write_char('{')?; // }
        inner.fmt_to(self)?;
        self.buf.write_char('}')?;
        Ok(())
    }
}

impl<'a, N> ExprFormatter<N> for Formatter<'a>
where
    N: Signed + DisplayWithContext,
    Expr<N>: HasPosExp + Inv<Output = Expr<N>> + Clone + Signed,
{
    type Error = Error;

    #[inline]
    fn get_buf(&mut self) -> &mut dyn fmt::Write {
        self.buf
    }

    fn fmt_in_parens(&mut self, inner: impl Formattable<N, Self>) -> Result<(), Self::Error> {
        self.buf.write_str(r"\left (")?; // )
        inner.fmt_to(self)?;
        self.buf.write_str(r"\right )")?; // )
        Ok(())
    }

    fn fmt_fn_call(
        &mut self,
        name: impl Formattable<N, Self>,
        inner: impl Formattable<N, Self>,
    ) -> Result<(), Self::Error> {
        self.buf.write_str(r"\mathrm{")?; // }
        name.fmt_to(self)?;
        self.buf.write_str("}")?;
        self.fmt_in_parens(inner)?;
        Ok(())
    }

    fn fmt_num(&mut self, num: &N) -> Result<(), Self::Error> {
        self.buf
            .write_str(&num.display_in(self.radix, self.config))
            .map_err(Error::from)
    }

    fn write_product_separator(&mut self) -> Result<(), Self::Error> {
        self.buf.write_str(r"cdot").map_err(Error::from)
    }

    fn fmt_frac(
        &mut self,
        numer: impl Iterator<Item = impl Formattable<N, Self>>,
        denom: impl Iterator<Item = impl Formattable<N, Self>>,
    ) -> Result<(), Self::Error> {
        self.buf.write_str(r"\frac{")?; // }
        self.fmt_frac_component(numer)?;
        self.buf.write_str("}{")?; // }
        self.fmt_frac_component(denom)?;
        self.buf.write_str("}")?;
        Ok(())
    }

    fn fmt_power(&mut self, base: &Expr<N>, exp: &Expr<N>) -> Result<(), Self::Error> {
        // TODO: roots
        self.buf.write_str("{")?; // }
        self.fmt(base)?;
        self.buf.write_str("}^{")?; // }
        self.fmt(exp)?;
        self.buf.write_str("}")?;
        Ok(())
    }

    fn fmt_log(&mut self, base: &Expr<N>, arg: &Expr<N>) -> Result<(), Self::Error> {
        self.buf.write_str(r"\log_{")?; // }
        self.fmt(base)?;
        self.buf.write_str(r"}{")?; // }
        self.fmt(arg)?;
        self.buf.write_str("}")?;
        Ok(())
    }

    // TODO: convert non-ASCII text to latex macros where possible
    fn fmt_var(&mut self, var: &str) -> Result<(), Self::Error> {
        if !var.is_ascii() {
            return Err(Error::NonAsciiVariable);
        }

        self.buf.write_str(var).map_err(Error::from)
    }

    fn fmt_const(&mut self, cnst: Const) -> Result<(), Self::Error> {
        self.buf
            .write_str(cnst.display_latex())
            .map_err(Error::from)
    }

    fn fmt_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        _units: AngleMeasure,
    ) -> Result<(), Self::Error> {
        self.fmt_latex_call(func, arg)
    }

    fn fmt_inv_trig(
        &mut self,
        func: impl Formattable<N, Self>,
        arg: &Expr<N>,
        _units: AngleMeasure,
    ) -> Result<(), Self::Error> {
        self.fmt_latex_call(func, arg)
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    use num::BigRational;

    #[test]
    fn test_single_frac() {
        assert_eq!(
            Expr::<BigRational>::from((5, 6)).display_latex(),
            r"\frac{5}{6}"
        );
    }
}
