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
    pub fn iappend_to_string(s: &mut String, n: usize) {
        s.push_str(&n.to_string());
    }

    #[inline(always)]
    pub fn iappend_write(s: &mut String, n: usize) {
        use std::fmt::Write;
        write!(s, "{}", n).ok();
    }

    #[inline(always)]
    pub fn iappendslice_to_string(s: &mut String, v: &[usize]) {
        for &n in v {
            s.push_str(&n.to_string());
            s.push(' ');
        }
    }

    #[inline(always)]
    pub fn iappendslice_write(s: &mut String, v: &[usize]) {
        use std::fmt::Write;
        for &n in v {
            write!(s, "{}", n).ok();
            s.push(' ');
        }
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

    #[inline(always)]
    pub fn iappend_07(s: &mut String, mut n: usize) {
        const MAX: usize = usize::ilog10(usize::MAX) as _;
        
        #[cfg(target_pointer_width = "64")]
        const _/* static assert */: [(); 19] = [(); MAX];

        //s.reserve(1 + MAX);

        unsafe {
            let buf = s.as_mut_vec();
            let mut push_unchecked = |byte| {
                let len = buf.len();
                std::ptr::write(buf.as_mut_ptr().add(len), byte);
                buf.set_len(len + 1);
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
        
    }

    #[inline(always)]
    pub fn iappendslice_07(s: &mut String, v: &[usize]) {
        const MAX: usize = usize::ilog10(usize::MAX) as _;
        
        #[cfg(target_pointer_width = "64")]
        const _/* static assert */: [(); 19] = [(); MAX];
        
        s.reserve((2 + MAX) * v.len());

        unsafe {
            let buf = s.as_mut_vec();
            let mut push_unchecked = |byte| {
                let len = buf.len();
                std::ptr::write(buf.as_mut_ptr().add(len), byte);
                buf.set_len(len + 1);
            };
            for mut n in v.iter().copied() {
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
                push_unchecked(b' ');
            }
        }
        
    }

    mod dec4le {
        macro_rules! invariant {
            ($expr: expr) => {
                debug_assert!($expr);
                if !($expr) {
                    #[allow(unused_unsafe)]
                    unsafe {
                        core::hint::unreachable_unchecked()
                    };
                }
            };
        }
        const D4: [u32; 10000] = {
            let (mut d4, mut i) = ([0u32; 10000], 0u32);
            while i < 10000 {
                let (dh, dl) = (i / 100, i % 100);
                d4[i as usize] = ((dl % 10) << 24) | ((dl / 10) << 16) | ((dh % 10) << 8) | (dh / 10) | 0x30303030;
                i += 1;
            }
            d4
        };
        #[inline(always)]
        unsafe fn raw_d4l(v: &mut *mut u8, x: u32) {
            unsafe {
                invariant!(x < 10000);
                match x {
                    0..=9 => {
                        **v = x as u8 | 0x30;
                        *v = v.add(1);
                    }
                    10..=99 => {
                        v.copy_from_nonoverlapping((D4[x as usize] >> 16).to_le_bytes().as_ptr(), 2);
                        *v = v.add(2);
                    }
                    100..=999 => {
                        v.copy_from_nonoverlapping((D4[x as usize] >> 8).to_le_bytes().as_ptr(), 3);
                        *v = v.add(3);
                    }
                    1000..=9999 => {
                        v.copy_from_nonoverlapping(D4[x as usize].to_le_bytes().as_ptr(), 4);
                        *v = v.add(4);
                    }
                    _ => core::hint::unreachable_unchecked(),
                }
            }
        }
        #[inline(always)]
        unsafe fn raw_d4(v: &mut *mut u8, x: u32) {
            unsafe {
                invariant!(x < 1_0000);
                v.copy_from_nonoverlapping(D4[x as usize].to_le_bytes().as_ptr(), 4);
                *v = v.add(4);
            }
        }
        #[inline(always)]
        unsafe fn raw_d8l(v: &mut *mut u8, x: u32) {
            unsafe {
                invariant!(x < 1_0000_0000);
                if x < 10000 {
                    raw_d4l(v, x);
                } else {
                    let (y0, y1) = (x / 1_0000, x % 1_0000);
                    raw_d4l(v, y0);
                    raw_d4(v, y1);
                }
            }
        }
        #[inline(always)]
        unsafe fn raw_d8(v: &mut *mut u8, x: u32) {
            unsafe {
                invariant!(x < 1_0000_0000);
                let (y0, y1) = (x / 1_0000, x % 1_0000);
                v.copy_from_nonoverlapping((((D4[y1 as usize] as u64) << 32) | (D4[y0 as usize] as u64)).to_le_bytes().as_ptr(), 8);
                *v = v.add(8);
            }
        }
        #[inline(always)]
        pub unsafe fn raw_u64(v: &mut *mut u8, x: u64) {
            unsafe {
                match x {
                    0..=9999_9999 => {
                        raw_d8l(v, x as u32);
                    }
                    1_0000_0000..=9999_9999_9999_9999 => {
                        let (z0, z1) = ((x / 1_0000_0000) as u32, (x % 1_0000_0000) as u32);
                        raw_d8l(v, z0);
                        raw_d8(v, z1);
                    }
                    1_0000_0000_0000_0000..=u64::MAX => {
                        let (y0, y1) = (
                            (x / 1_0000_0000_0000_0000) as u32,
                            x % 1_0000_0000_0000_0000,
                        );
                        let (z0, z1) = ((y1 / 1_0000_0000) as u32, (y1 % 1_0000_0000) as u32);
                        raw_d8l(v, y0);
                        raw_d8(v, z0);
                        raw_d8(v, z1);
                    }
                }
            }
        }
    }

    // Example of performance degradation when trying to implement std::fmt::Display
    pub struct U64Write(pub u64);
    impl std::fmt::Display for U64Write {
        #[inline(always)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe {
                let mut buf = [0u8; 20];
                let r = buf.as_mut_ptr();
                let mut p = r;
                dec4le::raw_u64(&mut p, self.0);
                f.write_str(std::str::from_utf8_unchecked(&buf[..(p.offset_from(r) as usize)]))
            }
        }
    }

    #[inline(always)]
    pub fn itoa_08d(n: usize) -> String {
        // Example of performance degradation when trying to implement std::fmt::Display
        U64Write(n as u64).to_string()
    }

    #[inline(always)]
    pub fn itoa_08(n: usize) -> String {
        let mut s = String::with_capacity(1 + (u64::ilog10(u64::MAX) as usize));
        unsafe {
            let v = s.as_mut_vec();
            let r = v.as_mut_ptr();
            let mut p = r;
            dec4le::raw_u64(&mut p, n as u64);
            v.set_len(p.offset_from(r) as usize);
        }
        s
    }

    #[inline(always)]
    pub fn iappend_08d(s: &mut String, n: usize) {
        // Example of performance degradation when trying to implement std::fmt::Display
        use std::fmt::Write;
        write!(s, "{}", U64Write(n as u64)).ok();
    }

    #[inline(always)]
    pub fn iappend_08(s: &mut String, n: usize) {
        unsafe {
            let v = s.as_mut_vec();
            let r = v.as_mut_ptr();
            let mut p = r.add(v.len());
            dec4le::raw_u64(&mut p, n as u64);
            v.set_len(p.offset_from(r) as usize);
        }
    }

    #[inline(always)]
    pub fn iappendslice_08d(s: &mut String, v: &[usize]) {
        // Example of performance degradation when trying to implement std::fmt::Display
        use std::fmt::Write;
        for &n in v.iter() {
            write!(s, "{}", U64Write(n as u64)).ok();
            if !(s.len() < s.capacity()) {
                unsafe { core::hint::unreachable_unchecked(); }
            }
            s.push(' ');
        }
    }

    #[inline(always)]
    pub fn iappendslice_08(s: &mut String, v: &[usize]) {
        s.reserve((2 + (u64::ilog10(u64::MAX) as usize)) * v.len());
        unsafe {
            let sv = s.as_mut_vec();
            let r = sv.as_mut_ptr();
            let mut p = r.add(sv.len());
            for &n in v.iter() {
                dec4le::raw_u64(&mut p, n as u64);
                *p = b' ';
                p = p.add(1);
            }
            sv.set_len(p.offset_from(r) as usize);
        }
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
                    b.iter(|| {
                        /*
                            reuse the same buffer and avoid redundant allocation during each iter

                            - `.as_mut_vec().set_len(0)` is very cheap at the cost
                            - `buf.{push, push_str}` just do `ptr::write` and `set_len` due to the enough capacity,
                              whitch means following `c_lib` overwrites existing bytes
                            - `Drop` impls of overwritten bytes are never called but this is not a problem
                              because it (`u8::drop`) is noop
                        */
                        unsafe {buf.as_mut_vec().set_len(0)}

                        c_lib(&mut buf)
                    });
                }
            )*
        }
    };
}
macro_rules! benchmark_append {
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
                    let push_sp_unchecked = |s: &mut String| {
                        unsafe {
                            let v = s.as_mut_vec();
                            if !(v.len() < v.capacity()) {
                                core::hint::unreachable_unchecked();
                            }
                            v.push(b' ');
                        }
                    };

                    let c_std = || {
                        let mut buf = String::with_capacity(v.len() * 21);
                        for s in v {
                            super::candiate::iappend_to_string(&mut buf, s);
                            push_sp_unchecked(&mut buf);
                        }
                        buf
                    };
                    let c_lib = || {
                        let mut buf = String::with_capacity(v.len() * 21);
                        for s in v {
                            super::candiate::$target(&mut buf, s);
                            push_sp_unchecked(&mut buf);
                        }
                        buf
                    };

                    assert_eq!(c_std(), c_lib());

                    b.iter(|| c_lib());
                }
            )*
        }
    };
}
macro_rules! benchmark_append_slice {
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

                    let c_std = || {
                        let mut buf = String::with_capacity(v.len() * 21);
                        super::candiate::iappendslice_to_string(&mut buf, &v);
                        buf
                    };
                    let c_lib = || {
                        let mut buf = String::with_capacity(v.len() * 21);
                        super::candiate::$target(&mut buf, &v);
                        buf
                    };

                    assert_eq!(c_std(), c_lib());

                    b.iter(|| c_lib());
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
    itoa_08d
    itoa_08
}
benchmark! {common_json_response_content_length: 0..(1 * 2_usize.pow(20))/* ~1MB */;
    itoa_to_string
    itoa_01
    itoa_02
    itoa_03
    itoa_05
    itoa_06
    itoa_07
    itoa_08d
    itoa_08
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
    itoa_08d
    itoa_08
}
benchmark! {max: 0..=usize::MAX;
    itoa_to_string
    itoa_01
    itoa_02
    itoa_03
    itoa_05
    itoa_06
    itoa_07
    itoa_08d
    itoa_08
}
benchmark_append! {once_a_small_http_response_content_length: 0..(1 * 2_usize.pow(10))/* ~1KB */;
    iappend_to_string
    iappend_write
    iappend_07
    iappend_08
    iappend_08d
}
benchmark_append! {once_common_json_response_content_length: 0..(1 * 2_usize.pow(20))/* ~1MB */;
    iappend_to_string
    iappend_write
    iappend_07
    iappend_08
    iappend_08d
}
benchmark_append! {once_http_response_content_length: 0..(10 * 2_usize.pow(30))/* ~10GB */;
    iappend_to_string
    iappend_write
    iappend_07
    iappend_08
    iappend_08d
}
benchmark_append! {once_max: 0..=usize::MAX;
    iappend_to_string
    iappend_write
    iappend_07
    iappend_08
    iappend_08d
}

benchmark_append_slice! {slice_a_small_http_response_content_length: 0..(1 * 2_usize.pow(10))/* ~1KB */;
    iappendslice_to_string
    iappendslice_write
    iappendslice_07
    iappendslice_08
    iappendslice_08d
}
benchmark_append_slice! {slice_common_json_response_content_length: 0..(1 * 2_usize.pow(20))/* ~1MB */;
    iappendslice_to_string
    iappendslice_write
    iappendslice_07
    iappendslice_08
    iappendslice_08d
}
benchmark_append_slice! {slice_http_response_content_length: 0..(10 * 2_usize.pow(30))/* ~10GB */;
    iappendslice_to_string
    iappendslice_write
    iappendslice_07
    iappendslice_08
    iappendslice_08d
}
benchmark_append_slice! {slice_max: 0..=usize::MAX;
    iappendslice_to_string
    iappendslice_write
    iappendslice_07
    iappendslice_08
    iappendslice_08d
}
