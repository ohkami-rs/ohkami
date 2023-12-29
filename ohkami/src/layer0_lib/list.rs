use std::mem::MaybeUninit;


pub(crate) struct List<T, const CAPACITY: usize> {
    pub(crate) list: [MaybeUninit<T>; CAPACITY],
    pub(crate) next: usize,
}

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    pub(crate) fn new() -> Self {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        Self {
            next: 0,
            list: unsafe { MaybeUninit::<[MaybeUninit<T>; CAPACITY]>::uninit().assume_init() },
        }
    }
}

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    #[inline] pub(crate) fn push(&mut self, element: T) {
        if self.next == CAPACITY {
            panic!("Buffer over flow");
        }
        self.list[self.next].write(element);
        self.next += 1;
    }
}


#[cfg(test)]
const _: () = {
    use crate::layer0_lib::Slice;

    use super::{CowSlice};

    impl<T: PartialEq, const CAPACITY: usize> PartialEq for List<T, CAPACITY> {
        fn eq(&self, other: &Self) -> bool {
            let n = self.next;
            if other.next != n {
                return false
            } else if n == 0 {
                return true
            }

            for i in 0..n {
                if unsafe {self.list[i].assume_init_ref() != other.list[i].assume_init_ref()} {
                    return false
                }
            }
            true
        }
    }

    impl<const LENGTH: usize, const CAPACITY: usize> From<[(&'static str, &'static str); LENGTH]> for List<(CowSlice, CowSlice), CAPACITY> {
        fn from(array: [(&'static str, &'static str); LENGTH]) -> Self {
            let mut this = Self::new(); for (key, val) in array {
                this.push((
                    unsafe {CowSlice::Ref(Slice::from_bytes(key.as_bytes()))},
                    unsafe {CowSlice::Ref(Slice::from_bytes(val.as_bytes()))},
                ))
            } this
        }
    }
};
