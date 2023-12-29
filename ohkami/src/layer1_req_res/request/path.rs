use crate::layer0_lib::{Slice, List};


const LIMIT: usize = 2;

pub(crate) struct Path {
    raw:    Slice,
    params: List<Slice, LIMIT>,
}

impl Path {
    #[inline] pub(crate) fn init() -> Self {
        Self { raw: Slice::null(), params: List::new() }
    }

    #[inline] pub(crate) unsafe fn from_request_bytes(bytes: &[u8]) -> Self {
        Self { raw: Slice::from_bytes(bytes), params: List::new() }
    }
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        Self { raw: unsafe {Slice::from_bytes(literal.as_bytes())}, params: List::new() }
    }

    #[inline] pub(crate) unsafe fn assume_one_param<'p>(self) -> Result<&'p str, &'static str> {
        let List { list, next } = self.params;
        (next >= 1)
            .then_some(into_str(list[0])?)
            .ok_or_else(|| "No path params found")
    }
    #[inline] pub(crate) unsafe fn assume_two_params<'p>(self) -> Result<(&'p str, &'p str), &'static str> {
        let List { list, next } = self.params;
        (next >= 2)
            .then_some((into_str(list[0])?, into_str(list[1])?))
            .ok_or_else(|| "No path params found")
    }

    #[inline] pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        self.raw.as_bytes()
    }
    #[inline] pub(crate) unsafe fn as_str(&self) -> &str {
        std::str::from_utf8(self.raw.as_bytes()).unwrap()
    }
}

#[inline] unsafe fn into_str<'p>(mu: std::mem::MaybeUninit<Slice>) -> Result<&'p str, &'static str> {
    std::str::from_utf8(mu.assume_init().as_bytes())
        .map_err(|_| "Path param is not UTF-8")
}
