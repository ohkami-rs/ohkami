#![feature(test)] extern crate test;


mod candiate {#![allow(unused)]
    #[inline(always)]
    pub fn itoa_lib(n: usize) -> String {
        ohkami_lib::num::itoa(n)
    }

    #[inline(always)]
    pub fn itoa_to_string(n: usize) -> String {
        n.to_string()
    }

    #[inline(always)]
    pub fn itoa_01(mut n: usize) -> String {
        let log10 = match usize::checked_ilog10(n) {
            Some(log10) => log10 as usize,
            None        => return String::from("0")
        };
        let len = 1 + log10;
        let mut digits = vec![0u8; len];
        {
            for i in 0..log10 {
                let d = 10_usize.pow((log10 - i) as u32);
                let (div, rem) = (n / d, n % d);
                *unsafe {digits.get_unchecked_mut(i as usize)} = b'0' + div as u8;
                n = rem;
            }
            *unsafe {digits.get_unchecked_mut(log10)} = b'0' + n as u8;
        }
        unsafe {String::from_utf8_unchecked(digits)}
    }

    #[inline(always)]
    pub fn itoa_02(mut n: usize) -> String {
        let log10 = match usize::checked_ilog10(n) {
            Some(log10) => log10 as usize,
            None        => return String::from("0")
        };
        let mut digits = vec![b'0'; 1 + log10];
        {
            for i in 0..log10 {
                let d = 10_usize.pow((log10 - i) as u32);
                let q = n / d;
                *unsafe {digits.get_unchecked_mut(i as usize)} += q as u8;
                n -= d * q;
            }
            *unsafe {digits.get_unchecked_mut(log10)} += n as u8;
        }
        unsafe {String::from_utf8_unchecked(digits)}
    }

    #[inline(always)]
    pub fn itoa_03(mut n: usize) -> String {
        const MAX: usize = usize::ilog10(usize::MAX) as _;

        const DIGITS: [usize; MAX+1] = [
            10_usize.pow(MAX as u32-0),
            10_usize.pow(MAX as u32-1),
            10_usize.pow(MAX as u32-2),
            10_usize.pow(MAX as u32-3),
            10_usize.pow(MAX as u32-4),
            10_usize.pow(MAX as u32-5),
            10_usize.pow(MAX as u32-6),
            10_usize.pow(MAX as u32-7),
            10_usize.pow(MAX as u32-8),
            10_usize.pow(MAX as u32-9),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-10),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-11),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-12),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-13),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-14),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-15),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-16),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-17),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-18),
            #[cfg(target_pointer_width="64")] 10_usize.pow(MAX as u32-19),
        ];

        let log10 = match usize::checked_ilog10(n) {
            Some(log10) => log10 as usize,
            None        => return String::from("0")
        };
        let mut digits = vec![b'0'; 1 + log10];
        {
            for i in 0..log10 {
                let d = *unsafe {DIGITS.get(MAX-log10+i).unwrap()};
                let q = n / d;
                *unsafe {digits.get_mut(i).unwrap()} += q as u8;
                n -= d * q;
            }
            *unsafe {digits.get_mut(log10).unwrap()} += n as u8;
        }
        unsafe {String::from_utf8(digits).unwrap()}
    }

    #[inline(always)]
    pub fn itoa_05(mut n: usize) -> String {
        let mut buf = Vec::with_capacity(usize::ilog10(usize::MAX) as _);

        {
            if n >= 10_usize.pow(1) {
                if n >= 10_usize.pow(2) {
                    if n >= 10_usize.pow(3) {
                        if n >= 10_usize.pow(4) {
                            if n >= 10_usize.pow(5) {
                                if n >= 10_usize.pow(6) {
                                    if n >= 10_usize.pow(7) {
                                        if n >= 10_usize.pow(8) {
                                            if n >= 10_usize.pow(9) {
                                                #[cfg(target_pointer_width="64")]
                                                if n >= 10_usize.pow(10) {
                                                    if n >= 10_usize.pow(11) {
                                                        if n >= 10_usize.pow(12) {
                                                            if n >= 10_usize.pow(13) {
                                                                if n >= 10_usize.pow(14) {
                                                                    if n >= 10_usize.pow(15) {
                                                                        if n >= 10_usize.pow(16) {
                                                                            if n >= 10_usize.pow(17) {
                                                                                if n >= 10_usize.pow(18) {
                                                                                    if n >= 10_usize.pow(19) {
                                                                                        buf.push(b'0' + (n / 10_usize.pow(19)) as u8);
                                                                                        n %= 10_usize.pow(19)
                                                                                    }
                                                                                    buf.push(b'0' + (n / 10_usize.pow(18)) as u8);
                                                                                    n %= 10_usize.pow(18)
                                                                                }
                                                                                buf.push(b'0' + (n / 10_usize.pow(17)) as u8);
                                                                                n %= 10_usize.pow(17)
                                                                            }
                                                                            buf.push(b'0' + (n / 10_usize.pow(16)) as u8);
                                                                            n %= 10_usize.pow(16)
                                                                        }
                                                                        buf.push(b'0' + (n / 10_usize.pow(15)) as u8);
                                                                        n %= 10_usize.pow(15)
                                                                    }
                                                                    buf.push(b'0' + (n / 10_usize.pow(14)) as u8);
                                                                    n %= 10_usize.pow(14)
                                                                }
                                                                buf.push(b'0' + (n / 10_usize.pow(13)) as u8);
                                                                n %= 10_usize.pow(13)
                                                            }
                                                            buf.push(b'0' + (n / 10_usize.pow(12)) as u8);
                                                            n %= 10_usize.pow(12)
                                                        }
                                                        buf.push(b'0' + (n / 10_usize.pow(11)) as u8);
                                                        n %= 10_usize.pow(11)
                                                    }
                                                    buf.push(b'0' + (n / 10_usize.pow(10)) as u8);
                                                    n %= 10_usize.pow(10)
                                                }
                                                buf.push(b'0' + (n / 10_usize.pow(9)) as u8);
                                                n %= 10_usize.pow(9)
                                            }
                                            buf.push(b'0' + (n / 10_usize.pow(8)) as u8);
                                            n %= 10_usize.pow(8)
                                        }
                                        buf.push(b'0' + (n / 10_usize.pow(7)) as u8);
                                        n %= 10_usize.pow(7)
                                    }
                                    buf.push(b'0' + (n / 10_usize.pow(6)) as u8);
                                    n %= 10_usize.pow(6)
                                }
                                buf.push(b'0' + (n / 10_usize.pow(5)) as u8);
                                n %= 10_usize.pow(5)
                            }
                            buf.push(b'0' + (n / 10_usize.pow(4)) as u8);
                            n %= 10_usize.pow(4)
                        }
                        buf.push(b'0' + (n / 10_usize.pow(3)) as u8);
                        n %= 10_usize.pow(3)
                    }
                    buf.push(b'0' + (n / 10_usize.pow(2)) as u8);
                    n %= 10_usize.pow(2)
                }
                buf.push(b'0' + (n / 10_usize.pow(1)) as u8);
                n %= 10_usize.pow(1)
            }
            buf.push(b'0' + n as u8)
        }

        unsafe {String::from_utf8_unchecked(buf)}
    }

    #[inline(always)]
    pub fn itoa_06(mut n: usize) -> String {
        let mut buf = Vec::<u8>::with_capacity(1 + usize::ilog10(usize::MAX) as usize);

        {
            macro_rules! push_unchecked {
                ($byte:expr) => {{
                    let len = buf.len();
                    std::ptr::write(buf.as_mut_ptr().add(len), $byte);
                    buf.set_len(len + 1);
                }};
            }

            if n >= 10_usize.pow(1) {
                if n >= 10_usize.pow(2) {
                    if n >= 10_usize.pow(3) {
                        if n >= 10_usize.pow(4) {
                            if n >= 10_usize.pow(5) {
                                if n >= 10_usize.pow(6) {
                                    if n >= 10_usize.pow(7) {
                                        if n >= 10_usize.pow(8) {
                                            if n >= 10_usize.pow(9) {
                                                #[cfg(target_pointer_width="64")]
                                                if n >= 10_usize.pow(10) {
                                                    if n >= 10_usize.pow(11) {
                                                        if n >= 10_usize.pow(12) {
                                                            if n >= 10_usize.pow(13) {
                                                                if n >= 10_usize.pow(14) {
                                                                    if n >= 10_usize.pow(15) {
                                                                        if n >= 10_usize.pow(16) {
                                                                            if n >= 10_usize.pow(17) {
                                                                                if n >= 10_usize.pow(18) {
                                                                                    if n >= 10_usize.pow(19) {
                                                                                        let q = n / 10_usize.pow(19);
                                                                                        unsafe {push_unchecked!(b'0' + q as u8)}
                                                                                        n -= 10_usize.pow(19) * q
                                                                                    }
                                                                                    let q = n / 10_usize.pow(18);
                                                                                    unsafe {push_unchecked!(b'0' + q as u8)}
                                                                                    n -= 10_usize.pow(18) * q
                                                                                }
                                                                                let q = n / 10_usize.pow(17);
                                                                                unsafe {push_unchecked!(b'0' + q as u8)}
                                                                                n -= 10_usize.pow(17) * q
                                                                            }
                                                                            let q = n / 10_usize.pow(16);
                                                                            unsafe {push_unchecked!(b'0' + q as u8)}
                                                                            n -= 10_usize.pow(16) * q
                                                                        }
                                                                        let q = n / 10_usize.pow(15);
                                                                        unsafe {push_unchecked!(b'0' + q as u8)}
                                                                        n -= 10_usize.pow(15) * q
                                                                    }
                                                                    let q = n / 10_usize.pow(14);
                                                                    unsafe {push_unchecked!(b'0' + q as u8)}
                                                                    n -= 10_usize.pow(14) * q
                                                                }
                                                                let q = n / 10_usize.pow(13);
                                                                unsafe {push_unchecked!(b'0' + q as u8)}
                                                                n -= 10_usize.pow(13) * q
                                                            }
                                                            let q = n / 10_usize.pow(12);
                                                            unsafe {push_unchecked!(b'0' + q as u8)}
                                                            n -= 10_usize.pow(12) * q
                                                        }
                                                        let q = n / 10_usize.pow(11);
                                                        unsafe {push_unchecked!(b'0' + q as u8)}
                                                        n -= 10_usize.pow(11) * q
                                                    }
                                                    let q = n / 10_usize.pow(10);
                                                    unsafe {push_unchecked!(b'0' + q as u8)}
                                                    n -= 10_usize.pow(10) * q
                                                }
                                                let q = n / 10_usize.pow(9);
                                                unsafe {push_unchecked!(b'0' + q as u8)}
                                                n -= 10_usize.pow(9) * q
                                            }
                                            let q = n / 10_usize.pow(8);
                                            unsafe {push_unchecked!(b'0' + q as u8)}
                                            n -= 10_usize.pow(8) * q
                                        }
                                        let q = n / 10_usize.pow(7);
                                        unsafe {push_unchecked!(b'0' + q as u8)}
                                        n -= 10_usize.pow(7) * q
                                    }
                                    let q = n / 10_usize.pow(6);
                                    unsafe {push_unchecked!(b'0' + q as u8)}
                                    n -= 10_usize.pow(6) * q
                                }
                                let q = n / 10_usize.pow(5);
                                unsafe {push_unchecked!(b'0' + q as u8)}
                                n -= 10_usize.pow(5) * q
                            }
                            let q = n / 10_usize.pow(4);
                            unsafe {push_unchecked!(b'0' + q as u8)}
                            n -= 10_usize.pow(4) * q
                        }
                        let q = n / 10_usize.pow(3);
                        unsafe {push_unchecked!(b'0' + q as u8)}
                        n -= 10_usize.pow(3) * q
                    }
                    let q = n / 10_usize.pow(2);
                    unsafe {push_unchecked!(b'0' + q as u8)}
                    n -= 10_usize.pow(2) * q
                }
                let q = n / 10_usize.pow(1);
                unsafe {push_unchecked!(b'0' + q as u8)}
                n -= 10_usize.pow(1) * q
            }
            unsafe {push_unchecked!(b'0' + n as u8)}
        }
        
        unsafe {String::from_utf8_unchecked(buf)}
    }

    #[inline(always)]
    pub fn itoa_07(mut n: usize) -> String {
        const MAX: usize = usize::ilog10(usize::MAX) as _;
        
        #[cfg(target_pointer_width = "64")]
        const _/* static assert */: [(); 19] = [(); MAX];
        
        let mut buf = Vec::<u8>::with_capacity(1 + MAX);

        {
            let mut push_unchecked = |byte| {
                let len = buf.len();
                unsafe {
                    std::ptr::write(buf.as_mut_ptr().add(len), byte);
                    buf.set_len(len + 1);
                }
            };

            macro_rules! unroll {
                () => {};
                ($digit:expr) => {unroll!($digit,)};
                ($digit:expr, $($tail:tt)*) => {
                    if $digit <= MAX && n >= 10_usize.pow($digit) {
                        unroll!($($tail)*);
                        let q = n / 10_usize.pow($digit);
                        push_unchecked(b'0' + q as u8);
                        n -= 10_usize.pow($digit) * q
                    }
                };
            }

            unroll!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19);

            push_unchecked(b'0' + n as u8);
        }
        
        unsafe {String::from_utf8_unchecked(buf)}
    }
}


macro_rules! benchmark {
    ($name:ident : $input_range:expr; $( $target:ident )*) => {
        mod $name {
            $(
                #[bench]
                fn $target(b: &mut test::Bencher) {            
                    let v: [usize; 10000] = {
                        use rand::prelude::*;
                        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(314159265358979);
                        std::array::from_fn(|_| rng.gen_range($input_range))
                    };

                    let c_std = |buf: &mut String| for n in v {
                        buf.push_str(&super::candiate::itoa_to_string(n));
                        buf.push(' ');
                    };
                    let c_lib = |buf: &mut String| for n in v {
                        buf.push_str(&super::candiate::$target(n));
                        buf.push(' ');
                    };

                    assert_eq!(
                        {let mut buf = String::new(); c_std(&mut buf); buf},
                        {let mut buf = String::new(); c_lib(&mut buf); buf}
                    );

                    let mut buf = String::with_capacity(v.len() * 21);
                    b.iter(|| c_lib(&mut buf));
                }
            )*
        }
    };
}
benchmark! {a_small_http_response_content_length: 0..(1 * 2_usize.pow(10))/* ~1KB */;
    itoa_to_string
    itoa_01
    itoa_02
    itoa_03
    itoa_05
    itoa_06
    itoa_07
}
benchmark! {common_json_response_content_length: 0..(1 * 2_usize.pow(20))/* ~1MB */;
    itoa_to_string
    itoa_01
    itoa_02
    itoa_03
    itoa_05
    itoa_06
    itoa_07
}
benchmark! {http_response_content_length: 0..(10 * 2_usize.pow(30))/* ~10GB
    usually large data more than 10GB are split into multiple responses or
    delivered via streaming, so `Content-Length`, length of a single
    response payload, will never exceed 10GB */;
    itoa_to_string
    itoa_01
    itoa_02
    itoa_03
    itoa_05
    itoa_06
    itoa_07
}
