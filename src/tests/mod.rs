mod ops;

use crate::{config::AngleMeasure, expr::constant::Const, Expr};
use num::{
    bigint::Sign,
    traits::{Pow, Zero},
    BigInt, BigRational, One, Signed,
};
use proptest::prelude::*;
use std::{
    fmt::Debug,
    iter::{Product, Sum},
    ops::{Div, Mul, Rem},
};

prop_compose! {
    fn arb_sign()(is_positive in any::<bool>()) -> Sign {
        if is_positive {
            Sign::Plus
        } else {
            Sign::Minus
        }
    }
}

prop_compose! {
    fn arb_bigint()(
        sign in arb_sign(),
        digits in prop::collection::vec(any::<u32>(), 0..4),
    ) -> BigInt {
        BigInt::from_slice(sign, &digits)
    }
}

prop_compose! {
    fn arb_bigrational()(
        numer in arb_bigint(),
        denom in arb_bigint().prop_filter("denominator should not be 0", |i| !i.is_zero()),
    ) -> BigRational {
        BigRational::from((numer, denom))
    }
}

fn arb_expr<N, S, F>(arb_n: F) -> impl Strategy<Value = Expr<N>>
where
    N: Debug + 'static,
    S: Strategy<Value = N> + 'static,
    F: Fn() -> S,
{
    let leaf = prop_oneof![
        any::<Const>().prop_map(Expr::Const),
        arb_n().prop_map(Expr::Num),
        any::<String>().prop_map(Expr::Var),
    ];
    leaf.prop_recursive(8, 128, 10, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..10).prop_map(Expr::Sum),
            prop::collection::vec(inner.clone(), 0..10).prop_map(Expr::Product),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| Expr::Power(Box::new(x), Box::new(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| Expr::Log(Box::new(x), Box::new(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| Expr::Mod(Box::new(x), Box::new(y))),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Sin(Box::new(x), m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Cos(Box::new(x), m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Tan(Box::new(x), m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Asin(Box::new(x), m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Acos(Box::new(x), m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| Expr::Atan(Box::new(x), m)),
        ]
    })
}

fn arb_simpl_expr<N, S, F>(arb_n: F) -> impl Strategy<Value = Expr<N>>
where
    N: 'static + PartialEq,
    S: Strategy<Value = N> + 'static,
    F: Fn() -> S,
    Expr<N>: Debug
        + Sum
        + Product
        + Pow<Expr<N>, Output = Expr<N>>
        + Mul<Output = Expr<N>>
        + Rem<Output = Expr<N>>
        + Div<Output = Expr<N>>
        + One
        + From<(i32, i32)>
        + Signed
        + PartialOrd
        + From<i32>
        + Clone,
{
    let leaf = prop_oneof![
        any::<Const>().prop_map(Expr::Const),
        arb_n().prop_map(Expr::Num),
        "[a-z]{1,4}".prop_map(Expr::Var),
    ];

    leaf.prop_recursive(8, 128, 10, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 2..10).prop_map(|v| v.into_iter().sum()),
            prop::collection::vec(inner.clone(), 2..10).prop_map(|v| v.into_iter().product()),
            (inner.clone(), inner.clone())
                .prop_filter("division by zero", |(x, y)| !(x.is_zero()
                    && y.is_negative()))
                .prop_map(|(x, y)| x.pow(y)),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| x.abs().log(y.abs())),
            (inner.clone(), inner.clone())
                .prop_filter("mod by 0", |(_, y)| !y.is_zero())
                .prop_map(|(x, y)| x.rem(y)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.generic_sin(m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.generic_cos(m)),
            (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.generic_tan(m)),
            // (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.asin(m)),
            // (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.acos(m)),
            // (inner.clone(), any::<AngleMeasure>()).prop_map(|(x, m)| x.atan(m)),
        ]
    })
}
