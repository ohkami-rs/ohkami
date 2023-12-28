use std::{mem::MaybeUninit, borrow::Cow};
use super::{CowSlice, Slice};


const LIMIT: usize = 8;

pub struct QueryParams {
    next:   usize,
    params: [MaybeUninit<(CowSlice, CowSlice)>; LIMIT],
} impl QueryParams {
    pub(crate) const fn new() -> Self {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        Self {
            next:   0,
            params: unsafe {
                MaybeUninit::<[MaybeUninit<(CowSlice, CowSlice)>; LIMIT]>::uninit().assume_init()
            },
        }
    }

    #[inline] pub(crate) fn iter<'q>(self) -> impl Iterator<Item = (CowSlice, CowSlice)> {
        self.params.into_iter()
            .take(self.next)
            .map(|mu| unsafe {mu.assume_init()})
    }

    #[inline] pub(crate) unsafe fn push_from_request_bytes(&mut self, key: &[u8], value: &[u8]) {
        let (key, value) = (Slice::from_bytes(key), Slice::from_bytes(value));
        self.params[self.next].write((CowSlice::Ref(key), CowSlice::Ref(value)));
        self.next += 1;
    }

    pub(crate) fn push(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) {
        fn into_cow_slice(c: impl Into<Cow<'static, str>>) -> CowSlice {
            match c.into() {
                Cow::Borrowed(str) => CowSlice::Ref(unsafe {Slice::from_bytes(str.as_bytes())}),
                Cow::Owned(string) => CowSlice::Own(string.into_bytes()),
            }
        }

        self.params[self.next].write((into_cow_slice(key), into_cow_slice(value)));
        self.next += 1;
    }
}
