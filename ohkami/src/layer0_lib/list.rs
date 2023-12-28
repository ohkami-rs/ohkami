use std::mem::MaybeUninit;


pub(crate) struct List<T, const CAPACITY: usize> {
    pub(crate) list: [MaybeUninit<T>; CAPACITY],
    pub(crate) next: usize,
}

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    pub(crate) fn new() -> Self {
        Self {
            list: std::array::from_fn(|_| MaybeUninit::uninit()),
            next: 0,
        }
    }
}

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    #[inline] pub(crate) fn append(&mut self, element: T) {
        if self.next == CAPACITY {
            panic!("Buffer over flow");
        } else {
            self.list[self.next].write(element);
            self.next += 1;
        }
    }

    #[inline] pub(crate) fn iter(&self) -> impl Iterator<Item = &'_ T> {
        let Self { list, next } = self;
        (&list[..*next])
            .into_iter()
            .map(|mu| unsafe {mu.assume_init_ref()})
    }
}
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
                this.append((
                    unsafe {CowSlice::Ref(Slice::from_bytes(key.as_bytes()))},
                    unsafe {CowSlice::Ref(Slice::from_bytes(val.as_bytes()))},
                ))
            } this
        }
    }
};
