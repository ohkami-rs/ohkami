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
