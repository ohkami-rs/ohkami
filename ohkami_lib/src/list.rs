use std::mem::MaybeUninit;


pub struct List<T, const CAPACITY: usize> {
    pub(crate) list: [MaybeUninit<T>; CAPACITY],
    pub(crate) next: usize,
}

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    pub const fn new() -> Self {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        Self {
            next: 0,
            list: unsafe { MaybeUninit::<[MaybeUninit<T>; CAPACITY]>::uninit().assume_init() },
        }
    }

    #[inline(always)] pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.list.get_unchecked(index).assume_init_ref()
    }

    /// SAFETY: The amount of calling `push`, `push_unchecked` for this `List` before
    /// is **LESS THAN** `CAPACITY`
    #[inline(always)] pub unsafe fn push_unchecked(&mut self, element: T) {
        self.list.get_unchecked_mut(self.next).write(element);
        self.next += 1;
    }

    #[inline] pub fn push(&mut self, element: T) {
        if self.next == CAPACITY {
            panic!("Buffer over flow");
        }

        // SAFETY: Here `self.next < CAPACITY`
        unsafe {self.push_unchecked(element)}
    }
}


const _: () = {
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

    impl<const N: usize, S: Into<T>, T> From<[(S, S); N]> for List<(T, T), N> {
        fn from(array: [(S, S); N]) -> Self {
            let mut this = Self::new();
            for (k, v) in array {
                this.push((k.into() , v.into()))
            }
            this
        }
    }
};
