use crate::{expr::Expr, tests::{arb_bigrational, arb_simpl_expr}};

use proptest::prelude::*;

macro_rules! x {
    () => {
        Expr::Var(String::from("x"))
    };
}

mod add {
    use num::Zero;

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
            assert!((expr.clone() - expr).is_zero());
        }
    }
}

mod mul {
    use num::{integer::Roots, traits::Pow, One, Zero};

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
    }
}
