#![feature(test)]

extern crate test;

use core::str::FromStr;
use rust_decimal::Decimal;

macro_rules! bench_decimal_op {
    ($name:ident, $op:tt, $y:expr) => {
        #[bench]
        fn $name(b: &mut ::test::Bencher) {
            let x = Decimal::from_str("2.01").unwrap();
            let y = Decimal::from_str($y).unwrap();
            b.iter(|| {
                let result = x $op y;
                ::test::black_box(result);
            });
        }
    }
}

macro_rules! bench_fold_op {
    ($name:ident, $op:tt, $init:expr, $count:expr) => {
        #[bench]
        fn $name(b: &mut ::test::Bencher) {
            fn fold(values: &[Decimal]) -> Decimal {
                let mut acc: Decimal = $init.into();
                for value in values {
                    acc = acc $op value;
                }
                acc
            }

            let values: Vec<Decimal> = test::black_box((1..$count).map(|i| i.into()).collect());
            b.iter(|| {
                let result = fold(&values);
                ::test::black_box(result);
            });
        }
    }
}

/* Add */
bench_decimal_op!(add_one, +, "1");
bench_decimal_op!(add_two, +, "2");
bench_decimal_op!(add_one_hundred, +, "100");
bench_decimal_op!(add_point_zero_one, +, "0.01");
bench_decimal_op!(add_negative_point_five, +, "-0.5");
bench_decimal_op!(add_pi, +, "3.1415926535897932384626433832");
bench_decimal_op!(add_negative_pi, +, "-3.1415926535897932384626433832");

bench_fold_op!(add_10k, +, 0, 10_000);

/* Sub */
bench_decimal_op!(sub_one, -, "1");
bench_decimal_op!(sub_two, -, "2");
bench_decimal_op!(sub_one_hundred, -, "100");
bench_decimal_op!(sub_point_zero_one, -, "0.01");
bench_decimal_op!(sub_negative_point_five, -, "-0.5");
bench_decimal_op!(sub_pi, -, "3.1415926535897932384626433832");
bench_decimal_op!(sub_negative_pi, -, "-3.1415926535897932384626433832");

bench_fold_op!(sub_10k, -, 5_000_000, 10_000);

/* Mul */
bench_decimal_op!(mul_one, *, "1");
bench_decimal_op!(mul_two, *, "2");
bench_decimal_op!(mul_one_hundred, *, "100");
bench_decimal_op!(mul_point_zero_one, *, "0.01");
bench_decimal_op!(mul_negative_point_five, *, "-0.5");
bench_decimal_op!(mul_pi, *, "3.1415926535897932384626433832");
bench_decimal_op!(mul_negative_pi, *, "-3.1415926535897932384626433832");

/* Div */
bench_decimal_op!(div_one, /, "1");
bench_decimal_op!(div_two, /, "2");
bench_decimal_op!(div_one_hundred, /, "100");
bench_decimal_op!(div_point_zero_one, /, "0.01");
bench_decimal_op!(div_negative_point_five, /, "-0.5");
bench_decimal_op!(div_pi, /, "3.1415926535897932384626433832");
bench_decimal_op!(div_negative_pi, /, "-3.1415926535897932384626433832");

bench_fold_op!(div_10k, /, Decimal::max_value(), 10_000);

/* Iteration */
struct DecimalIterator {
    count: usize,
}

impl DecimalIterator {
    fn new() -> DecimalIterator {
        DecimalIterator { count: 0 }
    }
}

impl Iterator for DecimalIterator {
    type Item = Decimal;

    fn next(&mut self) -> Option<Decimal> {
        self.count += 1;
        if self.count < 6 {
            Some(Decimal::new(314, 2))
        } else {
            None
        }
    }
}

#[bench]
fn iterator_individual(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut result = Decimal::new(0, 0);
        let iterator = DecimalIterator::new();
        for i in iterator {
            result += i;
        }
        ::test::black_box(result);
    });
}

#[bench]
fn iterator_sum(b: &mut ::test::Bencher) {
    b.iter(|| {
        let result: Decimal = DecimalIterator::new().sum();
        ::test::black_box(result);
    });
}

#[bench]
fn decimal_from_str(b: &mut test::Bencher) {
    let samples_strs = &[
        "3950.123456",
        "3950",
        "0.1",
        "0.01",
        "0.001",
        "0.0001",
        "0.00001",
        "0.000001",
        "1",
        "-100",
        "-123.456",
        "119996.25",
        "1000000",
        "9999999.99999",
        "12340.56789",
    ];

    b.iter(|| {
        for s in samples_strs {
            let result = Decimal::from_str(s).unwrap();
            test::black_box(result);
        }
    })
}

#[cfg(feature = "postgres")]
#[bench]
fn to_from_sql(b: &mut ::test::Bencher) {
    use bytes::BytesMut;
    use postgres::types::{FromSql, Kind, ToSql, Type};

    let samples_strs = &[
        "3950.123456",
        "3950",
        "0.1",
        "0.01",
        "0.001",
        "0.0001",
        "0.00001",
        "0.000001",
        "1",
        "-100",
        "-123.456",
        "119996.25",
        "1000000",
        "9999999.99999",
        "12340.56789",
    ];

    let samples: Vec<Decimal> = test::black_box(samples_strs.iter().map(|x| Decimal::from_str(x).unwrap()).collect());
    let t = Type::new("".into(), 0, Kind::Simple, "".into());
    let mut bytes: BytesMut = BytesMut::with_capacity(100).into();

    b.iter(|| {
        for _ in 0..100 {
            for sample in &samples {
                bytes.clear();
                sample.to_sql(&t, &mut bytes).unwrap();
                let result = Decimal::from_sql(&t, &bytes).unwrap();
                ::test::black_box(result);
            }
        }
    });
}

#[bench]
fn powi(b: &mut ::test::Bencher) {
    // These exponents have to be fairly small because multiplcation overflows easily
    let samples = &[
        (Decimal::from_str("36.7").unwrap(), 5),
        (Decimal::from_str("0.00000007").unwrap(), 5),
        (Decimal::from(2), 64),
        (Decimal::from_str("8819287.19276555").unwrap(), 3),
        (Decimal::from_str("-8819287.19276555").unwrap(), 3),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.0.powi(sample.1);
            ::test::black_box(result);
        }
    });
}

#[bench]
fn sqrt(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from_str("36.7").unwrap(),
        Decimal::from_str("0.00000007").unwrap(),
        Decimal::from(2),
        Decimal::from_str("8819287.19276555").unwrap(),
        Decimal::from_str("-8819287.19276555").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.sqrt();
            ::test::black_box(result);
        }
    });
}

#[bench]
fn exp(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from_str("3.7").unwrap(),
        Decimal::from_str("0.07").unwrap(),
        Decimal::from(2),
        Decimal::from_str("8.19").unwrap(),
        Decimal::from_str("-8.19").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.exp();
            ::test::black_box(result);
        }
    });
}

#[bench]
fn norm_cdf(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from_str("3.7").unwrap(),
        Decimal::from_str("0.007").unwrap(),
        Decimal::from(2),
        Decimal::from_str("1.19").unwrap(),
        Decimal::from_str("-1.19").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.norm_cdf();
            ::test::black_box(result);
        }
    });
}

#[bench]
fn norm_pdf(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from_str("3.7").unwrap(),
        Decimal::from_str("0.007").unwrap(),
        Decimal::from(2),
        Decimal::from_str("1.19").unwrap(),
        Decimal::from_str("-1.19").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.norm_pdf();
            ::test::black_box(result);
        }
    });
}

#[bench]
fn ln(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from_str("36.7").unwrap(),
        Decimal::from_str("0.00000007").unwrap(),
        Decimal::from(2),
        Decimal::from_str("8819287.19").unwrap(),
        Decimal::from_str("-8819287.19").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.ln();
            ::test::black_box(result);
        }
    });
}

#[bench]
fn erf(b: &mut ::test::Bencher) {
    let samples = &[
        Decimal::from(0),
        Decimal::from(1),
        Decimal::from_str("-0.98717").unwrap(),
        Decimal::from_str("0.07").unwrap(),
        Decimal::from_str("0.1111").unwrap(),
        Decimal::from_str("0.4").unwrap(),
    ];
    b.iter(|| {
        for sample in samples.iter() {
            let result = sample.erf();
            ::test::black_box(result);
        }
    });
}
