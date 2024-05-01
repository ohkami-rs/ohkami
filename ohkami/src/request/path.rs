use std::{borrow::Cow, mem::MaybeUninit};
use ohkami_lib::{percent_decode_utf8, Slice};


pub struct Path(
    MaybeUninit<PathInner>
);
pub(crate) struct PathInner {
    raw:    Slice,
    #[allow(unused)]
    params: Vec<Slice>,
}

const _: () = {
    impl Path {
        pub fn params(&self) -> impl Iterator<Item = Cow<str>> {
            unsafe {self.0.assume_init_ref()}
                .params.iter()
                .map(|slice| percent_decode_utf8(unsafe {slice.as_bytes()})
                .expect("Non UTF-8 path params"))
        }

        /// Get request path as `Cow::Borrowed(&str)` if it's not percent-encoded, or, if encoded,
        /// decode it into `Cow::Owned(String)`.
        #[inline]
        pub fn str(&self) -> Cow<str> {
            percent_decode_utf8(unsafe {self.0.assume_init_ref().raw.as_bytes()})
                .expect("Non UTF-8 path params")
        }
    }

    impl AsRef<str> for Path {
        #[inline]
        fn as_ref(&self) -> &str {
            let bytes = &unsafe {self.0.assume_init_ref().raw.as_bytes()};
            std::str::from_utf8(bytes).expect("Non UTF-8 path params")
        }
    }
    impl std::ops::Deref for Path {
        type Target = str;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.as_ref()
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

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
impl Path {
    #[inline]
    pub(crate) fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    #[inline(always)] pub(crate) fn from_request_bytes(bytes: &[u8]) -> Result<Self, crate::Response> {
        bytes.starts_with(b"/").then_some(())
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
        Ok(Self(MaybeUninit::new(PathInner {
            raw:    unsafe {Slice::new_unchecked(bytes.as_ptr(), len)},
            params: Vec::new(),
        })))
    }

    #[inline] pub(crate) unsafe fn push_param(&mut self, param: Slice) {
        self.0.assume_init_mut().params.push(param)
    }
    #[inline] pub(crate) unsafe fn assume_one_param<'p>(&self) -> &'p [u8] {
        self.0.assume_init_ref().params.get_unchecked(0).as_bytes()
    }
    #[inline] pub(crate) unsafe fn assume_two_params<'p>(&self) -> (&'p [u8], &'p [u8]) {
        (self.0.assume_init_ref().params.get_unchecked(0).as_bytes(), self.0.assume_init_ref().params.get_unchecked(1).as_bytes())
    }

    #[inline] pub(crate) unsafe fn normalized_bytes<'req>(&self) -> &'req [u8] {
        self.0.assume_init_ref().raw.as_bytes()
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(test)] impl Path {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        Self(MaybeUninit::new(PathInner {
            raw:    Slice::from_bytes(literal.as_bytes()),
            params: Vec::new(),
        }))
    }
}
