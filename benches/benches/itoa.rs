#![feature(test)] extern crate test;


mod candiate {#![allow(unused)]
    #[inline(always)]
    pub fn to_string(n: usize) -> String {
        n.to_string()
    }

    #[inline(always)]
    pub fn itoa(mut n: usize) -> String {
        ohkami_lib::num::itoa(n)
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
    itoa
}
