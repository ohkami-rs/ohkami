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
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    #[inline(always)] pub(crate) fn new(bytes: &[u8]) -> Self {
        Self(Slice::from_bytes(bytes))
    }

    /// SAFETY: The `QueryParams` is already **INITIALIZED**.
    #[inline(always)] pub(crate) fn parse<'q, T: serde::Deserialize<'q>>(
        &'q self
    ) -> Result<T, impl serde::de::Error> {
        ohkami_lib::serde_urlencoded::from_bytes(unsafe {self.0.as_bytes()})
    }

    /// Returns an iterator of maybe-percent-decoded (key, value).
    /// 
    /// SAFETY: The `QueryParams` is already **INITIALIZED**.
    #[inline] pub(crate) fn iter(&self) -> impl Iterator<
        Item = (Cow<'_, str>, Cow<'_, str>)
    > {
        #[inline(always)]
        fn decoded_utf8(maybe_encoded: &[u8]) -> Cow<'_, str> {
            match percent_decode(maybe_encoded) {
                Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
                Cow::Owned(vec)      => String::from_utf8_lossy(&vec).into_owned().into(),
            }
        }

        unsafe {self.0.as_bytes()}
            .split(|b| b==&b'&')
            .map(|kv| {
                let eq = kv.iter().position(|b| b==&b'=').expect("invalid query params: missing `=`");
                (
                    decoded_utf8(unsafe {kv.get_unchecked(..eq)}),
                    decoded_utf8(unsafe {kv.get_unchecked(eq+1..)})
                )
            })
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
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

const _: () = {
    impl std::fmt::Debug for QueryParams {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map().entries(self.iter()).finish()
        }
    }
};
