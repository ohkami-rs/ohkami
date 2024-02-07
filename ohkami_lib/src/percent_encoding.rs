//! Based on <https://github.com/servo/rust-url/tree/master/percent_encoding>; MIT.

use std::{str, slice, borrow::Cow};


#[inline(always)] pub fn percent_decode_utf8(input: &[u8]) -> Result<Cow<'_, str>, str::Utf8Error> {
    PercentDecode { bytes: input.iter() }.decode_utf8()
}
#[inline(always)] pub fn percent_decode(input: &[u8]) -> Cow<'_, [u8]> {
    PercentDecode { bytes: input.iter() }.into_cow()
}


#[derive(Clone)]
struct PercentDecode<'a> {
    bytes: slice::Iter<'a, u8>,
}

#[inline] fn after_percent_sign(iter: &mut slice::Iter<'_, u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let h = char::from(*cloned_iter.next()?).to_digit(16)?;
    let l = char::from(*cloned_iter.next()?).to_digit(16)?;
    *iter = cloned_iter;
    Some(h as u8 * 0x10 + l as u8)
}

impl<'a> Iterator for PercentDecode<'a> {
    type Item = u8;

    #[inline] fn next(&mut self) -> Option<u8> {
        self.bytes.next().map(|&byte| {
            if byte == b'%' {
                after_percent_sign(&mut self.bytes).unwrap_or(byte)
            } else {
                byte
            }
        })
    }
}

impl<'a> PercentDecode<'a> {
    #[inline] fn if_any(&self) -> Option<Vec<u8>> {
        let mut bytes_iter = self.bytes.clone();
        while bytes_iter.any(|&b| b == b'%') {
            if let Some(decoded_byte) = after_percent_sign(&mut bytes_iter) {
                let initial_bytes = self.bytes.as_slice();
                let unchanged_bytes_len = initial_bytes.len() - bytes_iter.len() - 3;
                let mut decoded = initial_bytes[..unchanged_bytes_len].to_owned();
                decoded.push(decoded_byte);
                decoded.extend(PercentDecode { bytes: bytes_iter });
                return Some(decoded);
            }
        }
        None
    }

    #[inline] fn decode_utf8(self) -> Result<Cow<'a, str>, str::Utf8Error> {
        match self.into_cow() {
            Cow::Borrowed(bytes) => match str::from_utf8(bytes) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(e),
            },
            Cow::Owned(bytes) => match String::from_utf8(bytes) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(e.utf8_error()),
            },
        }
    }

    #[inline(always)] fn into_cow(self) -> Cow<'a, [u8]> {
        match self.if_any() {
            Some(vec) => Cow::Owned(vec),
            None => Cow::Borrowed(self.bytes.as_slice()),
        }
    }
}


#[cfg(test)] mod test {
    #[test] fn test_percent_decode() {
        for (encoded, expected) in [
            // https://everything.curl.dev/http/post/url-encode
            (
                "John%20Doe%20%28Junior%29",
                "John Doe (Junior)",
            ),

            // https://developer.mozilla.org/ja/docs/Web/JavaScript/Reference/Global_Objects/encodeURI
            (
                "%D1%88%D0%B5%D0%BB%D0%BB%D1%8B",
                "шеллы",
            ),

            // https://qiita.com/magnolia_k_/items/ad90b2911c0382d27d5d
            (
                "%E5%90%89%E7%A5%A5%E5%AF%BApm",
                "吉祥寺pm",
            ),
            (
                "%E9%81%8B%E5%96%B6%E3%81%AF%3F",
                "運営は?",
            ),

            // https://help.sap.com/doc/saphelp_nw70/7.0.12/ja-JP/ce/1d3fc8da774366aa633a953f02a71a/content.htm
            (
                "%3cscript%20src=%22http%3a%2f%2fwww.badplace.com%2fnasty.js%22%3e%3c%2fscript%3e",
                "<script src=\"http://www.badplace.com/nasty.js\"></script>",
            ),
            (
                "user.name='bob%27%3b%20update%20logintable%20set%20password%3d%270wn3d%27%3b--%00",
                "user.name='bob'; update logintable set password='0wn3d';--\0",
            ),

            // https://zenn.dev/ymasutani/articles/b995a605dff3ff
            (
                "%60~%21%40%23%24%25%5E%26%2A%28%29-_%3D%2B%5B%7B%5D%7D%C2%A5%7C%3B%3A%27%2C.%3C%3E%3F%60",
                "`~!@#$%^&*()-_=+[{]}¥|;:',.<>?`",
            ),
            (
                "abcdefghijklmnopqrstuvwxyz1234567890",
                "abcdefghijklmnopqrstuvwxyz1234567890",
            ),
            (
                "%E3%81%82%E3%81%84%E3%81%86%E3%81%88%E3%81%8A%E3%80%8C%E3%80%8D%EF%BC%9F",
                "あいうえお「」？",
            ),
        ] {
            assert_eq!(
                &super::percent_decode_utf8(encoded.as_bytes()).unwrap(),
                expected
            );
        }
    }
}
