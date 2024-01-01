const ENCODER: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PADDING: u8       = b'=';


pub fn encode(src: impl AsRef<[u8]>) -> String {
    let src = src.as_ref();
    let src_len = src.len();

    if src_len == 0 {
        return String::new()
    }

    let mut dst = vec![0; (src_len + 2) / 3 * 4];

    let (mut di, mut si) = (0, 0);
    let n = (src_len / 3) * 3;  // `n` is `src_len - (src_len % 3)`
    while si < n {
        let val = (src[si+0] as usize)<<16 | (src[si+1] as usize)<<8 | (src[si+2] as usize);

        dst[di+0] = ENCODER[val>>18&0x3F];
    	dst[di+1] = ENCODER[val>>12&0x3F];
    	dst[di+2] = ENCODER[val>>6&0x3F];
    	dst[di+3] = ENCODER[val&0x3F];

        si += 3;
        di += 4;
    }

    let remain = src_len - si;  // `remain` is `src_len % 3`
    if remain == 0 {
        return (|| unsafe {String::from_utf8_unchecked(dst)})()
    }

    let mut val = (src[si+0] as usize) << 16;
    if remain == 2 {
        val |= (src[si+1] as usize) << 8;
    }

    dst[di+0] = ENCODER[val>>18&0x3F];
    dst[di+1] = ENCODER[val>>12&0x3F];

    match remain {
        2 => {
            dst[di+2] = ENCODER[val>>6&0x3F];
            dst[di+3] = PADDING;
        }
        1 => {
            dst[di+2] = PADDING;
            dst[di+3] = PADDING;
        }
        _ => unsafe {std::hint::unreachable_unchecked()}
    }

    unsafe {String::from_utf8_unchecked(dst)}
}

pub fn decode(encoded: impl AsRef<[u8]>) -> Vec<u8> {
    todo!()
}




#[cfg(test)] mod test {
    type Src     = &'static [u8];
    type Encoded = &'static str;

    const CASES: &[(Src, Encoded)] = &[
        // RFC 3548 examples
        (b"\x14\xfb\x9c\x03\xd9\x7e", "FPucA9l+"),
        (b"\x14\xfb\x9c\x03\xd9",     "FPucA9k="),
        (b"\x14\xfb\x9c\x03",         "FPucAw=="),

        // RFC 4648 examples
        (b"",       ""),
        (b"f",      "Zg=="),
        (b"fo",     "Zm8="),
        (b"foo",    "Zm9v"),
        (b"foob",   "Zm9vYg=="),
        (b"fooba",  "Zm9vYmE="),
        (b"foobar", "Zm9vYmFy"),

        // Wikipedia examples
        (b"sure.",    "c3VyZS4="),
        (b"sure",     "c3VyZQ=="),
        (b"sur",      "c3Vy"),
        (b"su",       "c3U="),
        (b"leasure.", "bGVhc3VyZS4="),
        (b"easure.",  "ZWFzdXJlLg=="),
        (b"asure.",   "YXN1cmUu"),
        (b"sure.",    "c3VyZS4="),
    ];

    #[test] fn test_encode() {
        for (src, encoded) in CASES {
            assert_eq!(super::encode(src), *encoded);
        }
    }

    #[test] fn test_decode() {
        for (src, encoded) in CASES {
            assert_eq!(*src, super::decode(encoded));
        }
    }
}

