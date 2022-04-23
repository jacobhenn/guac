use super::Expr;
use num::BigRational;
use std::{
    clone::Clone,
    ops::{Add, AddAssign},
};

impl Expr {
    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn terms(&self) -> Vec<&Expr> {
        match self {
            Expr::Sum(ts) => ts.iter().collect(),
            other => vec![other],
        }
    }

    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn terms_mut(&mut self) -> Vec<&mut Expr> {
        match self {
            Expr::Sum(ts) => ts.iter_mut().collect(),
            other => vec![other],
        }
    }

    /// Convert this expression into a list of its terms. e.g., turns `2+x+y` into `[2, x, y]`
    pub fn into_terms(self) -> Vec<Expr> {
        match self {
            Expr::Sum(ts) => ts,
            other => vec![other],
        }
    }

    /// Does this expression have the same variables and exponents as another expression?
    pub fn is_like_term(&self, rhs: &Expr) -> bool {
        let self_factors = self.factors();
        let rhs_factors = rhs.factors();
        rhs_factors
            .iter()
            .all(|f| f.is_num() || self_factors.contains(f))
            && self_factors
                .iter()
                .all(|f| f.is_num() || rhs_factors.contains(f))
    }

    /// Return the rational factor of this expression.
    pub fn coefficient(self) -> BigRational {
        self.into_factors()
            .into_iter()
            .filter_map(|t| t.num())
            .product()
    }

    /// Add two expressions. **If they are not like terms, this function will return an incorrect result**.
    pub fn combine_like_terms(self, rhs: Expr) -> Expr {
        let mut res = vec![Self::Num(self.clone().coefficient() + rhs.coefficient())];
        res.extend(self.into_factors().into_iter().filter(|f| !f.is_num()));

        let mut prod = Self::Product(res);
        prod.correct();
        prod
    }
}

impl Add for Expr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut self_terms = self.into_terms();
        for rhs_term in rhs.into_terms() {
            if let Some(self_term) = self_terms.iter_mut().find(|st| st.is_like_term(&rhs_term)) {
                *self_term = self_term.clone().combine_like_terms(rhs_term);
            } else {
                self_terms.push(rhs_term);
            }
        }

        let mut res = Self::Sum(self_terms);
        res.correct();
        res
    }
}

impl AddAssign for Expr {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}
