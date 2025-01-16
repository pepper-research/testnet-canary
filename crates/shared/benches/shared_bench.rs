#![feature(test)]

extern crate test;

#[cfg(test)]
mod bench_tests {
    use fixed::types::extra::U6;
    use fixed::FixedI128;
    use rand::prelude::*;
    use spicenet_shared::FastInt;
    use test::Bencher;

    #[bench]
    fn bench_fast_int_add(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FastInt::from(rng.gen::<(i64)>()));
            num2_arr.push(FastInt::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] + num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fixed_int_add(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
            num2_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] + num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_normal_int_add(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(i64)>());
            num2_arr.push(rng.gen::<(i64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] + num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fractional_int_add(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(f64)>());
            num2_arr.push(rng.gen::<(f64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] + num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fast_int_sub(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FastInt::from(rng.gen::<(i64)>()));
            num2_arr.push(FastInt::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] - num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fixed_int_sub(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
            num2_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] - num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_normal_int_sub(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(i64)>());
            num2_arr.push(rng.gen::<(i64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] - num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fractional_int_sub(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(f64)>());
            num2_arr.push(rng.gen::<(f64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] - num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fast_int_mul(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FastInt::from(rng.gen::<(i64)>()));
            num2_arr.push(FastInt::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] * num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fixed_int_mul(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
            num2_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] * num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_normal_int_mul(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(i64)>());
            num2_arr.push(rng.gen::<(i64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] * num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fractional_int_mul(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(f64)>());
            num2_arr.push(rng.gen::<(f64)>());
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] * num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fast_int_div(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FastInt::from(rng.gen::<(i64)>()));
            let mut num2_rng = rng.gen::<(i64)>();
            while num2_rng == 0 {
                let mut num2_rng = rng.gen::<(i64)>();
            }
            num2_arr.push(FastInt::from(num2_rng));
        }

        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] / num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fixed_int_div(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(FixedI128::<U6>::from(rng.gen::<(i64)>()));
            let mut num2_rng = rng.gen::<(i64)>();
            while num2_rng == 0 {
                let mut num2_rng = rng.gen::<(i64)>();
            }
            num2_arr.push(FixedI128::<U6>::from(num2_rng));
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] / num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_normal_int_div(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            num1_arr.push(rng.gen::<(i64)>());
            let mut num2_rng = rng.gen::<(i64)>();
            while num2_rng == 0 {
                let mut num2_rng = rng.gen::<(i64)>();
            }
            num2_arr.push(num2_rng);
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] / num2_arr[v];
            }
        });
    }

    #[bench]
    fn bench_fractional_int_div(b: &mut Bencher) {
        let mut rng = StdRng::seed_from_u64(69);
        let mut num1_arr = Vec::new();
        let mut num2_arr = Vec::new();
        for v in 0..100 {
            let mut num1_rng = rng.gen::<(f64)>();
            while num1_rng == 0.0 {
                let mut num1_rng = rng.gen::<(f64)>();
            }
            num1_arr.push(num1_rng);
            let mut num2_rng = rng.gen::<(f64)>();
            while num2_rng == 0.0 {
                let mut num2_rng = rng.gen::<(f64)>();
            }
            num2_arr.push(num2_rng);
        }
        b.iter(|| {
            for v in 0..100 {
                num1_arr[v] / num2_arr[v];
            }
        });
    }
}
