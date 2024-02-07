use std::{mem::MaybeUninit, borrow::Cow};
use ohkami_lib::percent_decode;
use super::{CowSlice, Slice};


const LIMIT: usize = 8;

pub struct QueryParams {
    next:   usize,
    /// `MaybeUninit<({percent decoded key}, {percent decoded value})>`
    params: [MaybeUninit<(CowSlice, CowSlice)>; LIMIT],
} impl QueryParams {
    #[inline] pub(crate) const fn new() -> Self {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        Self {
            next:   0,
            params: unsafe {
                MaybeUninit::<[MaybeUninit<(CowSlice, CowSlice)>; LIMIT]>::uninit().assume_init()
            },
        }
    }

    #[inline] pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        unsafe {self.params.get_unchecked(0..self.next)}.into_iter()
            .map(|mu| unsafe {
                let (k, v) = mu.assume_init_ref();
                (std::str::from_utf8(k.as_bytes()).unwrap(), std::str::from_utf8(v.as_bytes()).unwrap())
            })
    }
    #[inline] pub(crate) fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        let key = key.as_bytes();
        for kv in unsafe {self.params.get_unchecked(0..self.next)} {
            unsafe {
                let (k, v) = kv.assume_init_ref();
                if key == k.as_bytes() {
                    return Some((|| match v {
                        CowSlice::Ref(slice) => Cow::Borrowed(std::str::from_utf8(slice.as_bytes()).unwrap()),
                        CowSlice::Own(vec)   => Cow::Owned(String::from_utf8(vec.to_vec()).unwrap()),
                    })())
                }
            }
        }
        None
    }

    #[inline] pub(crate) unsafe fn push_from_request_slice(&mut self, key: Slice, value: Slice) {
        let (key, value) = (percent_decode(key.as_bytes()), percent_decode(value.as_bytes()));
        self.params.get_unchecked_mut(self.next).write((
            CowSlice::from_request_cow_bytes(key),
            CowSlice::from_request_cow_bytes(value),
        ));
        self.next += 1;
    }

    pub(crate) fn push(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) {
        fn into_cow_slice(c: impl Into<Cow<'static, str>>) -> CowSlice {
            match c.into() {
                Cow::Borrowed(str) => CowSlice::Ref(unsafe {Slice::from_bytes(str.as_bytes())}),
                Cow::Owned(string) => CowSlice::Own(string.into_bytes()),
            }
        }

        unsafe {self.params.get_unchecked_mut(self.next)}.write((into_cow_slice(key), into_cow_slice(value)));
        self.next += 1;
    }
}

const _: () = {
    impl PartialEq for QueryParams {
        fn eq(&self, other: &Self) -> bool {
            for (k, v) in self.iter() {
                if other.get(k) != Some(Cow::Borrowed(v)) {
                    return false
                }
            }
            true
        }
    }

    impl<const N: usize> From<[(&'static str, &'static str); N]> for QueryParams {
        fn from(kv: [(&'static str, &'static str); N]) -> Self {
            let mut this = QueryParams::new();
            for (k, v) in kv {
                this.push(k, v)
            }
            this
        }
    }
};
