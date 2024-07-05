#[inline]
pub fn hexized(n: usize) -> String {
    unsafe {String::from_utf8_unchecked(
        hexized_bytes(n).into()
    )}
}

#[inline(always)]
pub fn hexized_bytes(n: usize) -> [u8; std::mem::size_of::<usize>() * 2] {
    use std::mem::{size_of, transmute};

    unsafe {
        transmute::<_, [u8; size_of::<usize>() * 2]>(
            n.to_be_bytes().map(|byte| [byte>>4, byte&0b1111])
        ).map(|h| h + match h {
            0..=9   => b'0'-0,
            10..=15 => b'a'-10,
            _ => std::hint::unreachable_unchecked()
        })
    }
}

#[cfg(test)]
#[test] fn test_hexize() {
    for (n, expected) in [
        (1,   "1"),
        (9,   "9"),
        (12,  "c"),
        (16,  "10"),
        (42,  "2a"),
        (314, "13a"),
    ] {
        assert_eq!(hexized(n).trim_start_matches('0'), expected)
    }
}


#[inline]
pub fn itoa(mut n: usize) -> String {
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

#[cfg(test)]
#[test] fn test_itoa() {
    for n in [
        0,
        1,
        4,
        10,
        11,
        99,
        100,
        109,
        999,
        1000,
        10_usize.pow(usize::ilog10(usize::MAX)) - 1,
        10_usize.pow(usize::ilog10(usize::MAX)),
        usize::MAX - 1,
        usize::MAX,
    ] {
        assert_eq!(itoa(n), n.to_string())
    }
}
