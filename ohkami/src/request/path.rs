use ohkami_lib::Slice;

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use ohkami_lib::List;


#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
const LIMIT: usize = 2;

/// This doesn't handle percent encoding by itself.
pub(crate) struct Path {
    raw: Slice,
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) params: List<Slice, LIMIT>,
}

impl Path {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline] pub(crate) fn init() -> Self {
        Self {
            raw: Slice::null(),
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            params: List::new(),
        }
    }

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline] pub(crate) unsafe fn from_request_bytes(bytes: &[u8]) -> Self {
        debug_assert! {
            bytes.starts_with(b"/")
        }

        /*
            Strip trailing '/' **even when `bytes` is just `b"/"`**
            (then the bytes become b"" (empty bytes)).

            This suits to ohkami's radix router's searching algorithm and,
            while `Path::as_internal_bytes` directly returns the result bytes,
            `Path::as_bytes`, intended to be used by `Request::{pub fn path(&self)}`,
            returns `b"/"` if that bytes is `b"/"`.
        */
        let mut len = bytes.len();
        if *bytes.get_unchecked(len-1) == b'/' {len-=1};

        Self {
            raw:    Slice::new(bytes.as_ptr(), len),
            params: List::new(),
        }
    }

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline] pub(crate) unsafe fn assume_one_param<'p>(&self) -> &'p [u8] {
        self.params.get_unchecked(0).as_bytes()
    }
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline] pub(crate) unsafe fn assume_two_params<'p>(&self) -> (&'p [u8], &'p [u8]) {
        (self.params.get_unchecked(0).as_bytes(), self.params.get_unchecked(1).as_bytes())
    }

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline] pub(crate) unsafe fn as_internal_bytes<'req>(&self) -> &'req [u8] {
        self.raw.as_bytes()
    }
    #[inline] pub(crate) unsafe fn as_bytes<'req>(&self) -> &'req [u8] {
        match self.raw.as_bytes() {
            b""  => b"/",
            some => some,
        }
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(test)] impl Path {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        Self { raw: unsafe {Slice::from_bytes(literal.as_bytes())}, params: List::new() }
    }
}
