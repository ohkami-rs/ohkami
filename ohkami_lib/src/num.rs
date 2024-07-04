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
pub fn atoi(mut n: usize) -> String {
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

#[cfg(test)]
#[test] fn test_atoi() {
    for (n, expected) in [
        (0, "0"),
        (1, "1"),
        (4, "4"),
        (10, "10"),
        (11, "11"),
        (99, "99"),
        (100, "100"),
        (109, "109"),
        (999, "999"),
        (1000, "1000"),
    ] {
        assert_eq!(atoi(n), expected)
    }
}
