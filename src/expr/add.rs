use crate::expr::Expr;

use std::{
    clone::Clone,
    iter::Product,
    ops::{Add, AddAssign},
};

use num::{One, Zero, traits::Pow};

impl<N> Expr<N> {
    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn terms(&self) -> Vec<&Self> {
        match self {
            Self::Sum(ts) => ts.iter().collect(),
            other => vec![other],
        }
    }

    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn terms_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Self::Sum(ts) => ts.iter_mut().collect(),
            other => vec![other],
        }
    }

    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn into_terms(self) -> Vec<Self> {
        match self {
            Self::Sum(ts) => ts,
            other => vec![other],
        }
    }

    /// Does this expression have the same variables and exponents as another expression?
    pub fn is_like_term(&self, rhs: &Self) -> bool
    where
        N: PartialEq,
    {
        let self_factors = self.factors();
        let rhs_factors = rhs.factors();
        rhs_factors
            .iter()
            .all(|f| f.is_num() || self_factors.contains(f))
            && self_factors
                .iter()
                .all(|f| f.is_num() || rhs_factors.contains(f))
    }

    /// Return an immutable reference to the rational factor of this expression. If the rational factor is `1`, `None` will be returned, since the `1` isn't actually stored in the expression. **Expression must be `correct`ed**.
    pub fn coefficient(&self) -> Option<&N> {
        self.factors().into_iter().find_map(Self::num)
    }

    /// Return a mutable reference to the rational factor of this expression. If the rational factor is `1`, `None` will be returned, since the `1` isn't actually stored in the expression. **Expression must be `correct`ed**.
    pub fn coefficient_mut(&mut self) -> Option<&mut N> {
        self.factors_mut().into_iter().find_map(Self::num_mut)
    }

    /// Return the rational factor of this expression. If the rational factor is `1`, `None` will be returned, since the `1` isn't actually stored in the expression. **Expression must be `correct`ed**.
    pub fn into_coefficient(self) -> Option<N> {
        self.into_factors().into_iter().find_map(Self::into_num)
    }

    /// Add two expressions. **If they are not like terms, this function will return an incorrect result**.
    pub fn combine_like_terms(&mut self, rhs: Self)
    where
        N: One + Add<Output = N> + AddAssign + Clone,
        Self: From<i32>,
    {
        if let Some(c) = self.coefficient_mut() {
            *c += rhs.coefficient().cloned().unwrap_or_else(N::one);
        } else if let Some(c) = rhs.into_coefficient() {
            self.push_factor(Self::Num(c + N::one()));
        } else {
            match self {
                Self::Product(fs) => fs.push(Self::from(2)),
                other => *other = Self::Product(vec![Self::from(2), other.clone()]),
            }
        }
    }

    /// Naively add `rhs` to `self` without performing any simplifications. If `self` is a sum, append to existing term list.
    pub fn push_term(&mut self, rhs: Self)
    where
        Self: Clone,
    {
        match self {
            Self::Sum(ts) => ts.push(rhs),
            other => *other = Self::Sum(vec![rhs, other.clone()]),
        }
    }
}

impl<N> Add for Expr<N>
where
    Self: AddAssign,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<N> AddAssign for Expr<N>
where
    N: PartialEq + One + Add<Output = N> + AddAssign + Clone + Zero + for<'a> Product<&'a N>,
    Self: Clone + From<i32> + Pow<Self, Output = Self>,
{
    fn add_assign(&mut self, rhs: Self) {
        let self_terms = self.terms();
        let (like, unlike): (Vec<Self>, Vec<Self>) = rhs
            .into_terms()
            .into_iter()
            .partition(|t| self_terms.iter().any(|st| t.is_like_term(st)));

        for term in unlike {
            self.push_term(term);
        }

        let mut self_terms = self.terms_mut();
        for term in like {
            if let Some(self_term) = self_terms.iter_mut().find(|t| term.is_like_term(t)) {
                self_term.combine_like_terms(term);
            }
        }

        self.correct();
    }
}
