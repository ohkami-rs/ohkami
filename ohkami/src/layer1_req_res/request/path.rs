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
}

#[inline] unsafe fn into_str<'p>(mu: std::mem::MaybeUninit<Slice>) -> Result<&'p str, &'static str> {
    std::str::from_utf8(mu.assume_init().as_bytes())
        .map_err(|_| "Path param is not UTF-8")
}

/*
impl<T> List<T, 2> {
    #[inline] pub(crate) fn assume_init_first(self) -> T {
        if self.next == 0 {panic!("Called `assume_init_first` by `List` thats `next` is 0")}
        let [maybe_uninit_1, _] = self.list;
        unsafe {maybe_uninit_1.assume_init()}
    }
    #[inline] pub(crate) fn assume_init_extract(self) -> (T, T) {
        if self.next != 2 {panic!("Called `assume_init_extract` by `List` thats `next` doesn't equals to CAPACITY")}
        let [maybe_uninit_1, maybe_uninit_2] = self.list;
        unsafe {(maybe_uninit_1.assume_init(), maybe_uninit_2.assume_init())}
    }
}
*/
