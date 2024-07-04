#![feature(test)] extern crate test;


mod candiate {#![allow(unused)]
    pub fn to_string(n: usize) -> String {
        n.to_string()
    }

    #[inline(always)]
    pub fn atoi_01(mut n: usize) -> String {
        ohkami_lib::num::atoi(n)
    }
}


macro_rules! benchmark {
    ($( $target:ident )*) => {$(
        #[bench]
        fn $target(b: &mut test::Bencher) {
            b.iter(|| for n in 0..42 {
                let _ = candiate::$target(test::black_box(n));
            })
        }
    )*};
} benchmark! {
    to_string
    atoi_01
}
