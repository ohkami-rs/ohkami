pub fn encode(src: impl AsRef<[u8]>) -> String {
    encode_by(
        src.as_ref(),
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
        Some(b'='),
    )
}

pub fn encode_url(src: impl AsRef<[u8]>) -> String {
    encode_by(
        src.as_ref(),
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
        None,
    )
}

pub fn decode_url(encoded: impl AsRef<[u8]>) -> Vec<u8> {
    decode_by(
        encoded.as_ref(),
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
        None,
    )
}


fn encode_by(src: &[u8], encode_map: &[u8; 64], padding: Option<u8>) -> String {
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

        dst[di+0] = encode_map[val>>18&0x3F];
    	dst[di+1] = encode_map[val>>12&0x3F];
    	dst[di+2] = encode_map[val>>6&0x3F];
    	dst[di+3] = encode_map[val&0x3F];

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

    dst[di+0] = encode_map[val>>18&0x3F];
    dst[di+1] = encode_map[val>>12&0x3F];

    match remain {
        2 => {
            dst[di+2] = encode_map[val>>6&0x3F];
            if let Some(p) = padding {
                dst[di+3] = p;
            }
        }
        1 => if let Some(p) = padding {
            dst[di+2] = p;
            dst[di+3] = p;
        }
        _ => unsafe {std::hint::unreachable_unchecked()}
    }

    unsafe {String::from_utf8_unchecked(dst)}
}

fn decode_by(encoded: &[u8], encode_map: &[u8; 64], padding: Option<u8>) -> Vec<u8> {
    fn assemble64(n: [u8; 8]) -> Option<u64> {
        let [n1, n2, n3, n4, n5, n6, n7, n8] = n.map(<u8 as Into<u64>>::into);
        (n1|n2|n3|n4|n5|n6|n7|n8 != 0xff).then_some(
            n1<<58 | n2<<52 | n3<<46 | n4<<40 | n5<<34 | n6<<28 | n7<<22 | n8<<16
        )
    }
    fn assemble32(n: [u8; 4]) -> Option<u32> {
        let [n1, n2, n3, n4] = n.map(<u8 as Into<u32>>::into);
        (n1|n2|n3|n4 != 0xff).then_some(
            n1<<26 | n2<<20 | n3<<14 | n4<<8
        )
    }
    fn decode_quantum(dst: &mut Vec<u8>, encoded: &[u8], si: usize) -> (usize, usize) {
        todo!()
    }

    let mut decoded = vec![u8::default(); encoded.len()];
    if decoded.is_empty() {return decoded}

    let decode_map = {
        let mut map =[u8::MAX; 256];
        for (i, &byte) in encode_map.iter().enumerate() {
            map[byte as usize] = i as u8
        }
        map
    };

    let mut n  = 0;
    let mut si = 0;

    #[cfg(target_pointer_width = "64")]
    while encoded.len() - si >= 8 && decoded.len() - n >= 8 {
        let encoded2: [_; 8] = encoded[si..(si + 8)].try_into().unwrap();
        if let Some(dn) = assemble64(encoded2.map(|byte| decode_map[byte as usize])) {
            decoded[n..(n + 8)].copy_from_slice(&dn.to_be_bytes());
            n  += 6;
            si += 8;
        } else {
            let (new_si, n_inc) = decode_quantum(&mut decoded, encoded, si);
            n += n_inc;
            si = new_si;
        }
    }

    while encoded.len() - si >= 4 && decoded.len() - n >= 4 {
        let encoded2: [_; 4] = encoded[si..(si + 4)].try_into().unwrap();
        if let Some(dn) = assemble32(encoded2.map(|byte| decode_map[byte as usize])) {
            decoded[n..(n + 4)].copy_from_slice(&dn.to_be_bytes());
            n  += 3;
            si += 4;
        } else {
            let (new_si, n_inc) = decode_quantum(&mut decoded, encoded, si);
            n += n_inc;
            si = new_si;
        }
    }

    while si < encoded.len() {
        let (new_si, n_inc) = decode_quantum(&mut decoded, encoded, si);
        n += n_inc;
        si = new_si;
    }

    decoded.shrink_to_fit();
    decoded
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
}

