use crate::layer0_lib::{Slice, List};


const LIMIT: usize = 2;

/// This doesn't handle percent encoding by itself.
pub(crate) struct Path {
    raw:               Slice,
    pub(crate) params: List<Slice, LIMIT>,
}

impl Path {
    #[inline] pub(crate) fn init() -> Self {
        Self { raw: Slice::null(), params: List::new() }
    }

    #[inline] pub(crate) unsafe fn from_request_bytes(bytes: &[u8]) -> Self {
        Self { raw: Slice::from_bytes(bytes), params: List::new() }
    }

    #[inline] pub(crate) unsafe fn assume_one_param<'p>(&self) -> &'p [u8] {
        self.params.list.get_unchecked(0).assume_init_ref().as_bytes()
    }
    #[inline] pub(crate) unsafe fn assume_two_params<'p>(&self) -> (&'p [u8], &'p [u8]) {
        (
            self.params.list.get_unchecked(0).assume_init_ref().as_bytes(),
            self.params.list.get_unchecked(1).assume_init_ref().as_bytes(),
        )
    }

    #[inline] pub(crate) unsafe fn as_bytes<'req>(&self) -> &'req [u8] {
        self.raw.as_bytes()
    }
}

#[cfg(test)] impl Path {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        Self { raw: unsafe {Slice::from_bytes(literal.as_bytes())}, params: List::new() }
    }
}
