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
