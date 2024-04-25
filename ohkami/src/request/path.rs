use ohkami_lib::Slice;


/// This doesn't handle percent encoding by itself.
pub(crate) struct Path {
    raw:    Slice,
    #[allow(unused)]
    params: Vec<Slice>,
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
impl Path {
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
        
        Ok(Self {
            raw:    unsafe {Slice::new_unchecked(bytes.as_ptr(), len)},
            params: Vec::new(),
        })
    }

    #[inline] pub(crate) fn push_param(&mut self, param: Slice) {
        self.params.push(param)
    }
    #[inline] pub(crate) unsafe fn assume_one_param<'p>(&self) -> &'p [u8] {
        self.params.get_unchecked(0).as_bytes()
    }
    #[inline] pub(crate) unsafe fn assume_two_params<'p>(&self) -> (&'p [u8], &'p [u8]) {
        (self.params.get_unchecked(0).as_bytes(), self.params.get_unchecked(1).as_bytes())
    }

    #[inline] pub(crate) unsafe fn as_internal_bytes<'req>(&self) -> &'req [u8] {
        self.raw.as_bytes()
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(test)] impl Path {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        Self {
            raw:    Slice::from_bytes(literal.as_bytes()),
            params: Vec::new(),
        }
    }
}

impl Path {
    #[inline] pub(crate) unsafe fn as_bytes<'req>(&self) -> &'req [u8] {
        match self.raw.as_bytes() {
            b""  => b"/",
            some => some,
        }
    }
}
