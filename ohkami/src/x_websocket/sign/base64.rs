/* https://github.com/golang/go/blob/master/src/encoding/base64/base64.go */

const ENCODER: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PADDING: u8       = b'=';

pub struct Base64<
    const SRC_SIZE: usize,
    #[cfg(feature="nightly")] const DST_SIZE: usize = {(SRC_SIZE + 2) / 3 * 4},
    #[cfg(feature="nightly")] const SRC_SIZE_REM3_0: bool = {SRC_SIZE % 3 == 0},
    #[cfg(feature="nightly")] const SRC_SIZE_REM3_1: bool = {SRC_SIZE % 3 == 1},
    #[cfg(feature="nightly")] const SRC_SIZE_REM3_2: bool = {SRC_SIZE % 3 == 2},
>;

#[cfg(feature="nightly")] impl<
    const SRC_SIZE: usize,
    const DST_SIZE: usize,
    const SRC_SIZE_REM3_0: bool,
    const SRC_SIZE_REM3_1: bool,
    const SRC_SIZE_REM3_2: bool,
> Base64<SRC_SIZE, DST_SIZE, SRC_SIZE_REM3_0, SRC_SIZE_REM3_1, SRC_SIZE_REM3_2> {
    pub fn encode(src: [u8; SRC_SIZE]) -> String {
        if SRC_SIZE == 0 {// may deleted by compiler when `SRC_SIZE` is not 0
            return String::new()
        }

        #[cfg(feature="nightly")]
        let mut dst = vec![0; DST_SIZE];

        let (mut di, mut si) = (0, 0);
        let n = (SRC_SIZE / 3) * 3;  // `n` is `SRC_SIZE - (SRC_SIZE % 3)`
        while si < n {
            let val = (src[si+0] as usize)<<16 | (src[si+1] as usize)<<8 | (src[si+2] as usize);

            dst[di+0] = ENCODER[val>>18&0x3F];
	    	dst[di+1] = ENCODER[val>>12&0x3F];
	    	dst[di+2] = ENCODER[val>>6&0x3F];
	    	dst[di+3] = ENCODER[val&0x3F];

            si += 3;
            di += 4;
        }
        
        if SRC_SIZE_REM3_0 {// may deleted by compiler when `SRC_SIZE` is not a multiple of 3
            return (|| unsafe {String::from_utf8_unchecked(dst)})()
        }

        let mut val = (src[si+0] as usize) << 16;
        if SRC_SIZE_REM3_2 {// may be deleted by compiler when `SRC_SIZE` is congruent to 2 mod 3
            val |= (src[si+1] as usize) << 8;
        }

        dst[di+0] = ENCODER[val>>18&0x3F];
        dst[di+1] = ENCODER[val>>12&0x3F];

        if SRC_SIZE_REM3_2 {// may be deleted by compiler when `SRC_SIZE` is congruent to 2 mod 3
            dst[di+2] = ENCODER[val>>6&0x3F];
            dst[di+3] = PADDING;
        }
        if SRC_SIZE_REM3_1 {// may be deleted by compiler when `SRC_SIZE` is congruent to 1 mod 3
            dst[di+2] = PADDING;
            dst[di+3] = PADDING;
        }

        unsafe {String::from_utf8_unchecked(dst)}
    }
}


#[cfg(not(feature="nightly"))] impl<
    const SRC_SIZE: usize,
> Base64<SRC_SIZE> {
    pub fn encode(src: [u8; SRC_SIZE]) -> String {
        if SRC_SIZE == 0 {// may deleted by compiler when `SRC_SIZE` is not 0
            return String::new()
        }

        let mut dst = vec![0; (SRC_SIZE + 2) / 3 * 4];

        let (mut di, mut si) = (0, 0);
        let n = (SRC_SIZE / 3) * 3;  // `n` is `SRC_SIZE - (SRC_SIZE % 3)`
        while si < n {
            let val = (src[si+0] as usize)<<16 | (src[si+1] as usize)<<8 | (src[si+2] as usize);

            dst[di+0] = ENCODER[val>>18&0x3F];
	    	dst[di+1] = ENCODER[val>>12&0x3F];
	    	dst[di+2] = ENCODER[val>>6&0x3F];
	    	dst[di+3] = ENCODER[val&0x3F];

            si += 3;
            di += 4;
        }

        let remain = SRC_SIZE - si;  // `remain` is `SRC_SIZE % 3`
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
}

//}

//impl<const SRC_SIZE: usize> Base64<SRC_SIZE> {
//    pub fn encode(src: [u8; SRC_SIZE]) -> String {
//        if src.len() == 0 {
//            return String::new()
//        }
//
//        let mut dst = vec![0; (src.len() + 2) / 3 * 4];
//
//        let (mut di, mut si) = (0, 0);
//        let n = (SRC_SIZE / 3) * 3;  // `n` is `SRC_SIZE - (SRC_SIZE % 3)`
//        while si < n {
//            let val = (src[si+0] as usize)<<16 | (src[si+1] as usize)<<8 | (src[si+2] as usize);
//
//            dst[di+0] = ENCODER[val>>18&0x3F];
//	    	dst[di+1] = ENCODER[val>>12&0x3F];
//	    	dst[di+2] = ENCODER[val>>6&0x3F];
//	    	dst[di+3] = ENCODER[val&0x3F];
//
//            si += 3;
//            di += 4;
//        }
//
//        #[cfg(feature="nightly")] if SRC_SIZE_REM3_0 {// may deleted by compiler when `SRC_SIZE` is not a multiple of 3
//            return (|| unsafe {String::from_utf8_unchecked(dst)})()
//        }
//
//        #[cfg(not(feature="nightly"))] let remain = SRC_SIZE - si;  // `remain` is `SRC_SIZE % 3`
//        #[cfg(not(feature="nightly"))] {
//            if remain == 0 {return (|| unsafe {String::from_utf8_unchecked(dst)})()}
//        }
//
//        let mut val = (src[si+0] as usize) << 16;
//        #[cfg(feature="nightly")] if SRC_SIZE_REM3_2 {// may be deleted by compiler when `SRC_SIZE` is congruent to 2 mod 3
//            val |= (src[si+1] as usize) << 8;
//        }
//        #[cfg(not(feature="nightly"))] if remain == 2 {
//            val |= (src[si+1] as usize) << 8;
//        }
//
//        dst[di+0] = ENCODER[val>>18&0x3F];
//        dst[di+1] = ENCODER[val>>12&0x3F];
//
//        #[cfg(feature="nightly")] if SRC_SIZE_REM3_2 {// may be deleted by compiler when `SRC_SIZE` is congruent to 2 mod 3
//            dst[di+2] = ENCODER[val>>6&0x3F];
//            dst[di+3] = PADDING;
//        }
//        #[cfg(feature="nightly")] if SRC_SIZE_REM3_1 {// may be deleted by compiler when `SRC_SIZE` is congruent to 1 mod 3
//            dst[di+2] = PADDING;
//            dst[di+3] = PADDING;
//        }
//        #[cfg(not(feature="nightly"))] match remain {
//            2 => {
//                dst[di+2] = ENCODER[val>>6&0x3F];
//                dst[di+3] = PADDING;
//            }
//            1 => {
//                dst[di+2] = PADDING;
//                dst[di+3] = PADDING;
//            }
//            _ => unsafe {std::hint::unreachable_unchecked()}
//        }
//
//        unsafe {String::from_utf8_unchecked(dst)}
//    }
//}
//