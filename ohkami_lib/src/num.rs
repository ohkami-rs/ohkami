#[inline]
pub fn hexize(n: usize) -> String {
    use std::{mem, ptr};

    let mut hex = String::with_capacity(mem::size_of::<usize>() * 2);
    unsafe {
        for char_byte in mem::transmute::<_, [u8; mem::size_of::<usize>() * 2]>(
            n.to_be_bytes().map(|byte| [byte>>4, byte&(8+4+2+1)])
        ).map(|h| h + match h {
            0..=9   => b'0'-0,
            10..=15 => b'a'-10,
            _ => std::hint::unreachable_unchecked()
        }) {
            let hex = hex.as_mut_vec(); {
                let len = hex.len();
                ptr::write(hex.as_mut_ptr().add(len), char_byte);
                hex.set_len(len + 1);
            }
        }
    }

    hex
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
        assert_eq!(hexize(n).trim_start_matches('0'), expected)
    }
}
