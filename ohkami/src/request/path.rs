use std::{borrow::Cow, mem::MaybeUninit};
use ohkami_lib::{percent_decode_utf8, Slice};


pub struct Path(
    MaybeUninit<PathInner>
);
pub(crate) struct PathInner {
    raw:    Slice,
    params: Params,
}
struct Params {
    next: usize,
    list: [MaybeUninit<Slice>; Self::LIMIT]
}
impl Params {
    const LIMIT: usize = 2;
}

const _: () = {
    impl Params {
        fn iter(&self) -> impl Iterator<Item = &Slice> {
            (0..self.next).map(|i| unsafe {
                self.list
                    .get_unchecked(i)
                    .assume_init_ref()
            })
        }
    }

    impl Path {
        pub fn params(&self) -> impl Iterator<Item = Cow<str>> {
            unsafe {self.0.assume_init_ref()}
                .params.iter()
                .map(|slice| percent_decode_utf8(unsafe {slice.as_bytes()})
                .expect("Non UTF-8 path params"))
        }

        /// Get request path as `Cow::Borrowed(&str)` if it's not percent-encoded, or,
        /// decode it into `Cow::Owned(String)` if encoded in the original request.
        #[inline]
        pub fn str(&self) -> Cow<str> {
            let bytes = unsafe {self.0.assume_init_ref().raw.as_bytes()};
            if bytes.is_empty() {return Cow::Borrowed("/")}
            percent_decode_utf8(bytes).expect("Non UTF-8 path params")
        }

        #[inline] pub(crate) unsafe fn assume_one_param<'p>(&self) -> &'p [u8] {
            self.0.assume_init_ref().params.list.get_unchecked(0).assume_init_ref().as_bytes()
        }
        #[inline] pub(crate) unsafe fn assume_two_params<'p>(&self) -> (&'p [u8], &'p [u8]) {
            (
                self.0.assume_init_ref().params.list.get_unchecked(0).assume_init_ref().as_bytes(),
                self.0.assume_init_ref().params.list.get_unchecked(1).assume_init_ref().as_bytes()
            )
        }
    }

    impl std::ops::Deref for Path {
        type Target = str;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }
    impl AsRef<str> for Path {
        #[inline]
        fn as_ref(&self) -> &str {
            let bytes = &unsafe {self.0.assume_init_ref().raw.as_bytes()};
            if bytes.is_empty() {return "/"}
            std::str::from_utf8(bytes).expect("Non UTF-8 path params")
        }
    }

    impl std::fmt::Debug for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Cow<str> as std::fmt::Debug>::fmt(&self.str(), f)
        }
    }
    impl std::fmt::Display for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Cow<str> as std::fmt::Display>::fmt(&self.str(), f)
        }
    }
};

#[cfg(feature="__rt__")]
const _: () = {
    impl Params {
        const fn init() -> Self {
            Params { next: 0, list: [const {MaybeUninit::uninit()}; Params::LIMIT] }
        }
        
        #[inline(always)]
        fn push(&mut self, param: Slice) {
            #[cfg(debug_assertions)] {
                assert!(self.next < Self::LIMIT);
            }
            unsafe {self.list
                .get_unchecked_mut(self.next)
                .write(param);
            }
            self.next += 1;
        }
    }
    
    impl Path {
        pub(crate) const fn uninit() -> Self {
            Self(MaybeUninit::uninit())
        }

        #[inline(always)]
        pub(crate) fn init_with_request_bytes(&mut self, bytes: &[u8]) -> Result<(), crate::Response> {
            (bytes.first() == Some(&b'/')).then_some(())
                .ok_or_else(crate::Response::NotImplemented)?;

            /*
            Strip trailing '/' **even when `bytes` is just `b"/"`**
            (then the bytes become b"" (empty bytes)).

            This suits to ohkami's radix router's searching algorithm and,
            while `Path::as_internal_bytes` directly returns the result bytes,
            `Path::as_bytes`, intended to be used by `Request::{pub fn path(&self)}`,
            returns `b"/"` if that bytes is `b"/"`.
            */
            let mut len = bytes.len();
            if unsafe {*bytes.get_unchecked(len-1) == b'/'} {len -= 1};

            #[allow(unused_unsafe/* I don't know why but rustc sometimes put warnings to this unsafe as unnecessary */)]
            Ok({self.0.write(PathInner {
                raw:    unsafe {Slice::new_unchecked(bytes.as_ptr(), len)},
                params: Params::init(),
            });})
        }

        #[inline] pub(crate) unsafe fn push_param(&mut self, param: Slice) {
            self.0.assume_init_mut().params.push(param)
        }

        #[inline] pub(crate) unsafe fn normalized_bytes<'req>(&self) -> &'req [u8] {
            self.0.assume_init_ref().raw.as_bytes()
        }
    }
    
    impl Path {
        #[cfg(all(feature="__rt_native__", feature="DEBUG", test))]
        pub(crate) fn from_literal(literal: &'static str) -> Self {
            // SAFETY: `literal` is 'static
            unsafe {Self::from_str_unchecked(literal)}
        }
    
        #[allow(unused)]
        /// SAFETY: `s` outlives the actual (used) lifetime of `Path`
        pub(crate) unsafe fn from_str_unchecked(s: &str) -> Self {
            Self(MaybeUninit::new(PathInner {
                raw:    Slice::from_bytes(s.trim_end_matches('/').as_bytes()),
                params: Params::init(),
            }))
        }
    }
};
    