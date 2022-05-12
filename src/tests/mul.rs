use num::{bigint::RandBigInt, BigRational};
use rand;

use crate::expr::Expr;

// fn rand_bigint() -> BigInt {
//     let mut rng = rand::thread_rng();
//     rng.gen_bigint(64)
// }

fn rand_bigrational() -> BigRational {
    let mut rng = rand::thread_rng();
    let n = rng.gen_bigint(64);
    let m = rng.gen_bigint(64);
    BigRational::new(n, m)
}

#[test]
fn rational_mul() {
    for _ in 0..500 {
        let x = rand_bigrational();
        let y = rand_bigrational();
        let p = x.clone() * y.clone();

        let ex = Expr::Num(x);
        let ey = Expr::Num(y);
        let ep = ex * ey;

        assert_eq!(ep, Expr::Num(p))
    }
}

#[test]
fn rational_div() {
    for _ in 0..500 {
        let x = rand_bigrational();
        let y = rand_bigrational();
        let p = x.clone() / y.clone();

        let ex = Expr::Num(x);
        let ey = Expr::Num(y);
        let ep = ex / ey;

        assert_eq!(ep, Expr::Num(p))
    }
}
