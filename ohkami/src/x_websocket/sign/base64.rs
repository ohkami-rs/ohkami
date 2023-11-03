/* https://github.com/golang/go/blob/master/src/encoding/base64/base64.go */

const ENCODER: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PADDING: char     = '=';

const SIZE_FROM_SHA1: usize = 28;

pub fn encode_sha1_to_base64(sha1_bytes: [u8; super::sha1::SIZE]) -> [u8; SIZE_FROM_SHA1] {
    let mut dst = [0; SIZE_FROM_SHA1];

    let (mut di, mut si) = (0, 0);
    let n = (super::sha1::SIZE / 3) * 3;
    while si < n {
        let val = (sha1_bytes[si+0] as usize)<<16 | (sha1_bytes[si+1] as usize)<<8 | (sha1_bytes[si+2] as usize);

        dst[di+0] = ENCODER[val>>18&0x3F];
		dst[di+1] = ENCODER[val>>12&0x3F];
		dst[di+2] = ENCODER[val>>6&0x3F];
		dst[di+3] = ENCODER[val&0x3F];

        si += 3;
        di += 4;
    }

    let remain = super::sha1::SIZE - si;
    /* unreachable because `si` is a multiple of 3 and `sha1::SIZE` is 20 */
    // if remain == 0 {return dst}

    let mut val = (sha1_bytes[si+0] as usize) << 16;
    if remain == 2 {
        val |= (sha1_bytes[si+1] as usize) << 8;
    }

    dst[di+0] = ENCODER[val>>18&0x3F];
    dst[di+1] = ENCODER[val>>12&0x3F];

    match remain {
        2 => {
            dst[di+2] = ENCODER[val>>6&0x3F];
            dst[di+3] = b'=';
        }
        1 => {
            dst[di+2] = b'=';
            dst[di+3] = b'=';
        }
        _ => unsafe {std::hint::unreachable_unchecked()}
    }

    dst
}
