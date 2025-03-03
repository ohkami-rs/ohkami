use std::borrow::Cow;
use ohkami_lib::percent_decode;
use super::Slice;


#[derive(PartialEq)]
pub struct QueryParams(
    /// raw bytes of query params with leading '?' cut
    /// 
    /// ex) name=ohkami&type=framework
    Slice
);

impl QueryParams {
    #[cfg(feature="__rt__")]
    #[inline(always)] pub(crate) const fn new(bytes: &[u8]) -> Self {
        Self(Slice::from_bytes(bytes))
    }

    #[inline(always)] pub fn parse<'q, T: serde::Deserialize<'q>>(
        &'q self
    ) -> Result<T, impl serde::de::Error> {
        ohkami_lib::serde_urlencoded::from_bytes(unsafe {self.0.as_bytes()})
    }

    pub fn iter(&self) -> impl Iterator<
        Item = (Cow<'_, str>, Cow<'_, str>)
    > {
        #[inline(always)]
        fn decoded_utf8(maybe_encoded: &[u8]) -> Cow<'_, str> {
            match percent_decode(maybe_encoded) {
                Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
                Cow::Owned(vec)      => String::from_utf8_lossy(&vec).into_owned().into(),
            }
        }

        let bytes = unsafe {self.0.as_bytes()};
        (if bytes.is_empty() {None} else {Some(
            bytes
            .split(|b| b == &b'&')
            .filter_map(|kv| {
                match kv.iter().position(|b| b == &b'=') {
                    None => {
                        #[cfg(debug_assertions)] {
                            if kv.is_empty() {
                                crate::WARNING!("skipping an invalid query param: trailing `&`");
                            } else {
                                crate::WARNING!("skipping an invalid query param: missing `=`");
                            }
                        }
                        None
                    }
                    Some(0) => {
                        #[cfg(debug_assertions)] {
                            crate::WARNING!("skipping an invalid query param: empty key");
                        }
                        None
                    }
                    Some(n) => Some((
                        decoded_utf8(unsafe {kv.get_unchecked(..n)}),
                        decoded_utf8(unsafe {kv.get_unchecked(n+1..)})
                    ))
                }
            })
        )}).into_iter().flatten()
    }
}

const _: () = {
    impl std::fmt::Debug for QueryParams {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map().entries(self.iter()).finish()
        }
    }
};

#[cfg(feature="__rt_native__")]
#[cfg(test)]
const _: () = {
    impl<const N: usize> From<[(&'static str, &'static str); N]> for QueryParams {
        fn from(kvs: [(&'static str, &'static str); N]) -> Self {
            use ohkami_lib::percent_encode;

            let raw = kvs.into_iter()
                .map(|(k, v)| format!(
                    "{}={}",
                    percent_encode(k),
                    percent_encode(v),
                ))
                .collect::<Vec<_>>()
                .join("&");

            QueryParams::new(Box::leak(raw.into_boxed_str()).as_bytes())
        }
    }
};

#[cfg(not(feature="rt_worker"))]
#[cfg(test)]
#[test] fn query_iter() {
    let case = QueryParams(Slice::from_bytes(b"abc=def&xyz=123"));
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
        ("xyz".into(), "123".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b"abc=def&xyz=%E4%B8%80%E4%BA%8C%E4%B8%89"));
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
        ("xyz".into(), "一二三".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b"abc=def&xyz="));
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
        ("xyz".into(), "".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b"abc=def&xyz"));
    // skipping an invalid query param: missing `=`
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b"abc=def&"));
    // skipping an invalid query param: trailing `&`
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b"abc=def"));
    assert_eq!(case.iter().collect::<Vec<_>>(), [
        ("abc".into(), "def".into()),
    ]);

    let case = QueryParams(Slice::from_bytes(b""));
    assert_eq!(case.iter().collect::<Vec<_>>(), []);
}
