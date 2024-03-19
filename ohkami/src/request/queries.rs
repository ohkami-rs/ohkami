use std::mem::MaybeUninit;
use std::borrow::Cow;
use ohkami_lib::percent_decode;
use super::Slice;


pub struct QueryParams(
    /// raw bytes of query params with leading '?' cut
    /// 
    /// ex) name=ohkami&type=framework
    MaybeUninit<Slice>
);

impl QueryParams {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline(always)] pub(crate) fn new(bytes: &[u8]) -> Self {
        Self(MaybeUninit::new(unsafe {Slice::from_bytes(bytes)}))
    }

    /// SAFETY: The `QueryParams` is already **INITIALIZED**.
    #[inline(always)] pub(crate) unsafe fn parse<'q, T: serde::Deserialize<'q>>(&'q self) -> Result<T, impl serde::de::Error> {
        ohkami_lib::serde_urlencoded::from_bytes(self.0.assume_init_ref().as_bytes())
    }

    /// Returns an iterator of maybe-percent-decoded (key, value).
    /// 
    /// SAFETY: The `QueryParams` is already **INITIALIZED**.
    #[inline] pub(crate) unsafe fn iter(&self) -> impl Iterator<
        Item = (Cow<'_, str>, Cow<'_, str>)
    > {
        #[inline(always)]
        fn decoded_utf8(maybe_encoded: &[u8]) -> Cow<'_, str> {
            match percent_decode(maybe_encoded) {
                Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
                Cow::Owned(vec)      => String::from_utf8_lossy(&vec).into_owned().into(),
            }
        }

        self.0.assume_init_ref().as_bytes()
            .split(|b| b==&b'&')
            .map(|kv| {
                let (k, v) = kv.split_at(
                    kv.iter().position(|b| b==&b'=').expect("invalid query params: missing `=`")
                );
                (decoded_utf8(k), decoded_utf8(v))
            })
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(test)]
const _: () = {
    impl PartialEq for QueryParams {
        fn eq(&self, other: &Self) -> bool {
            let (this, other) = unsafe {(
                self.iter().collect::<Vec<_>>(),
                other.iter().collect::<Vec<_>>()
            )};
            for kv in this {
                if other.iter().find(|o_kv| o_kv == &&kv).is_none() {
                    return false
                }
            }
            true
        }
    }

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
