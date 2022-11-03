use crate::{
    config::AngleMeasure,
    expr::Expr,
    tests::{arb_bigrational, arb_simpl_expr},
};

use num::{
    integer::Roots,
    traits::{Inv, Pow},
    BigRational, One, Zero,
};

use std::ops::Neg;

use proptest::prelude::*;

macro_rules! x {
    () => {
        Expr::Var(String::from("x"))
    };
}

mod add {
    use super::*;

    prop_compose! {
        fn any_addable_i32()(n in (i32::MIN / 2)..=(i32::MAX / 2)) -> i32 {
            n
        }
    }

    proptest! {
        #[test]
        fn behaves_like_regular_addition(x in any_addable_i32(), y in any_addable_i32()) {
            let ex = Expr::Num(x);
            let ey = Expr::Num(y);
            let e_sum = Expr::Num(x + y);
            assert_eq!(ex + ey, e_sum);
        }

        #[test]
        fn sum_behaves_like_regular_addition(x in any_addable_i32(), y in any_addable_i32()) {
            let ex = Expr::Num(x);
            let ey = Expr::Num(y);
            let e_sum = Expr::Num(x + y);
            assert_eq!(vec![ex, ey].into_iter().sum::<Expr<_>>(), e_sum);
        }

        #[test]
        // n*x + m*x == (n+m)*x
        fn combine_like_terms(x in any_addable_i32(), y in any_addable_i32()) {
            let ex = Expr::Num(x) * x!();
            let ey = Expr::Num(y) * x!();
            let ex_plus_ey = ex + ey;
            let e_sum = Expr::Num(x + y);
            assert_eq!(ex_plus_ey.clone().coefficient().copied(), Some(x + y));
            assert_eq!(ex_plus_ey.clone() / x!(), e_sum);
            assert_eq!(ex_plus_ey, e_sum * x!());
        }

        #[test]
        // x - x == 0
        fn subtract_from_self(expr in arb_simpl_expr(arb_bigrational)) {
            dbg!(&expr);
            assert!((expr.clone() - expr).is_zero());
        }

        #[test]
        fn add_then_sub(
            x in arb_simpl_expr(arb_bigrational),
            y in arb_simpl_expr(arb_bigrational),
        ) {
            assert_eq!((x.clone() + y.clone()) - y, x);
        }
    }
}

mod mul {
    use super::*;

    prop_compose! {
        fn any_mulable_i32()(n in -i32::MAX.sqrt()..=i32::MAX.sqrt()) -> i32 {
            n
        }
    }

    proptest! {
        #[test]
        fn behaves_like_regular_multiplication(x in any_mulable_i32(), y in any_mulable_i32()) {
            let ex = Expr::Num(x);
            let ey = Expr::Num(y);
            let e_prod = Expr::Num(x * y);
            assert_eq!(ex * ey, e_prod);
        }

        #[test]
        fn product_behaves_like_regular_multiplication(
            x in any_mulable_i32(),
            y in any_mulable_i32()
        ) {
            let ex = Expr::Num(x);
            let ey = Expr::Num(y);
            let e_prod = Expr::Num(x * y);
            assert_eq!(vec![ex, ey].into_iter().product::<Expr<_>>(), e_prod);
        }

        #[test]
        // x^n * x^m == x^(n+m)
        fn combine_like_factors(x in any_mulable_i32(), y in any_mulable_i32()) {
            let ex = x!().pow(Expr::Num(x));
            let ey = x!().pow(Expr::Num(y));
            let ex_times_ey = ex * ey;
            let e_sum = Expr::Num(x + y);
            assert_eq!(ex_times_ey.clone().exponent().cloned(), Some(e_sum.clone()));
            assert_eq!(ex_times_ey, x!().pow(e_sum));
        }

        #[test]
        // x / x == 1
        fn divide_by_self(expr in arb_simpl_expr(arb_bigrational)) {
            prop_assume!(!expr.is_zero());
            assert!((expr.clone() / expr).is_one());
        }

        #[test]
        fn double_negation(expr in arb_simpl_expr(arb_bigrational)) {
            assert!(expr.clone().neg().neg() == expr);
        }

        // unfortunately, due to multiplication always distributing, this will not always be the
        // case.
        // #[test]
        // fn mul_then_div(
        //     x in arb_simpl_expr(arb_bigrational),
        //     y in arb_simpl_expr(arb_bigrational),
        // ) {
        //     assert_eq!((x.clone() * y.clone()) / y, x);
        // }
    }
}

mod pow {
    use super::*;

    prop_compose! {
        fn arb_perfect_root()(e in 2..=8u32)(
            (b, e) in (0..i32::MAX.nth_root(e)).prop_map(move |b| (b, e))
        ) -> (i32, u32) {
            (b, e)
        }
    }

    proptest! {
        #[test]
        // b, e ∈ ℕ => (b ** e) ** (1/e) == b
        fn integer_roots((b, e) in arb_perfect_root()) {
            let eb = Expr::from(b);
            let ee = Expr::<BigRational>::from(e as i32);
            let e_pow = Expr::from(b.pow(e));
            assert_eq!(e_pow.clone().pow(ee.inv()), eb);
        }

        #[test]
        fn double_inversion(expr in arb_simpl_expr(arb_bigrational)) {
            prop_assume!(!expr.is_zero());
            assert!(expr.clone().inv().inv() == expr);
        }
    }
}

mod trig {
    use super::*;

    #[test]
    #[allow(unused_must_use)]
    // test that none of the obvious boundary conditions cause a stack overflow on the
    // possibly-recursive trig methods
    fn boundaries() {
        for n in 0..4 {
            let n = Expr::<BigRational>::from(n);
            n.clone().generic_sin(AngleMeasure::Turn);
            n.clone().generic_cos(AngleMeasure::Turn);
            n.generic_tan(AngleMeasure::Turn);
        }
    }
}
