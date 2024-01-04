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
    fn decode_quantum(
        dst: &mut [u8],
        encoded: &[u8],
        mut si: usize,
        decode_map: &[u8; 256],
        padding: Option<u8>,
    ) -> (/*new si*/usize, /*n increase*/usize) {
        let mut d_len = 4;
        let mut d_buf = [u8::default(); 4];

        let mut i = 0;
        while i < d_buf.len() {
            if encoded.len() == si {
                if i == 0 {
                    return (si, 0)
                } else if i == 1 || padding.is_some() {
                    unreachable!("Illegal base64 data at input byte {}", si - i)
                }

                d_len = i;
                break
            }

            let input = encoded[si];
            si += 1;

            let output = decode_map[input as usize];
            if output != 0xff {
                d_buf[i] = output;
                continue
            }

            if matches!(input, b'\r' | b'\n') {
                i -= 1;
                continue
            }

            if padding != Some(input) {
                unreachable!("Illegal base64 data at input byte {}", si - 1)
            }

            /* We've reached the end and there's padding */
            match i {
                0 | 1 => unreachable!("Illegal base64 data at input byte {}: incorrect padding", si - 1),
                2 => {/* "==" is expected, the first "=" is already consumed.  */
                    /* skip over newlines */
                    while si < encoded.len() && matches!(encoded[si], b'\r' | b'\n') {si += 1}

                    if si == encoded.len() {
                        unreachable!("Illegal base64 data at input byte {}: not enough padding", encoded.len())
                    } else if padding != Some(encoded[si]) {
                        unreachable!("Illegal base64 data at input byte {}: incorrect padding", si - 1)
                    }

                    si += 1
                }
                _ => ()
            }

            /* skip over newlines */
            while si < encoded.len() && matches!(encoded[si], b'\r' | b'\n') {si += 1}
            if si < encoded.len() {
                unreachable!("Illegal base64 data at input byte {}: trailing garbage", si)
            }
            d_len = i;
            break;

            #[allow(unreachable_code)] {i += 1}
        }

        let val = (d_buf[0] as usize)<<18 | (d_buf[1] as usize)<<12 | (d_buf[2] as usize)<<6 | (d_buf[3] as usize);
        (d_buf[2], d_buf[1], d_buf[0]) = ((val>>0) as u8, (val>>8) as u8, (val>>16) as u8);
        match d_len {
            4 => {
                dst[2] = d_buf[2];
                d_buf[2] = 0;
            }
            3 => {
                dst[1] = d_buf[1];
                d_buf[1] = 0;
            }
            _ => ()
        }
        dst[0] = d_buf[0];

        (si, d_len - 1)
    }

    // ==================================================

    let mut decoded = {
        let max_len = encoded.len() / 4 * 3 +
            padding.is_none().then_some(encoded.len() % 4 * 6 / 8).unwrap_or(0);
        vec![u8::default(); max_len]
    };
    if decoded.is_empty() {return decoded}

    let decode_map = {
        let mut map =[0xff; 256];
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
            let (new_si, n_inc) = decode_quantum(&mut decoded[n..], encoded, si, &decode_map, padding);
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
            let (new_si, n_inc) = decode_quantum(&mut decoded[n..], encoded, si, &decode_map, padding);
            n += n_inc;
            si = new_si;
        }
    }

    while si < encoded.len() {
        let (new_si, n_inc) = decode_quantum(&mut decoded[n..], encoded, si, &decode_map, padding);
        n += n_inc;
        si = new_si;
    }

    decoded.truncate(n);
    decoded
}




#[cfg(test)] mod test {
    type Src     = &'static [u8];
    type Encoded = &'static str;

    const CASES: &[(Src, Encoded)] = &[
        // RFC 3548 examples
        //(b"\x14\xfb\x9c\x03\xd9\x7e", "FPucA9l+"),
        //(b"\x14\xfb\x9c\x03\xd9",     "FPucA9k="),
        //(b"\x14\xfb\x9c\x03",         "FPucAw=="),
        // RFC 4648 examples
        (b"",       ""),
        //(b"f",      "Zg=="),
        //(b"fo",     "Zm8="),
        (b"foo",    "Zm9v"),
        //(b"foob",   "Zm9vYg=="),
        //(b"fooba",  "Zm9vYmE="),
        //(b"foobar", "Zm9vYmFy"),
        // Wikipedia examples
        //(b"sure.",    "c3VyZS4="),
        //(b"sure",     "c3VyZQ=="),
        //(b"sur",      "c3Vy"),
        //(b"su",       "c3U="),
        //(b"leasure.", "bGVhc3VyZS4="),
        //(b"easure.",  "ZWFzdXJlLg=="),
        //(b"asure.",   "YXN1cmUu"),
        //(b"sure.",    "c3VyZS4="),
    ];

    #[test] fn test_encode() {
        for (src, encoded) in CASES {
            assert_eq!(super::encode(src), *encoded);
        }
    }

    #[test] fn test_decode() {
        fn decode(encoded: &[u8]) -> Vec<u8> {
            super::decode_by(
                encoded,
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
                Some(b'='),
            )
        }

        for (original, encoded) in CASES {
            let (left, right) = (decode(encoded.as_bytes()), original);
            if left != *right {
                panic!("\n\
                  \0 left: `{}`\n\
                    right: `{}`\n\
                ", left.escape_ascii(), right.escape_ascii())
            }
        }
    }
}

