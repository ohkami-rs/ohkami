type Byte = u8;

const NULL: Byte = Byte::MAX;

pub(crate) struct ByteArrayMap<const N: usize, Value> {
    /// using `u8` instead of `usize` to save memory space,
    /// with implicitly limiting the capacity to 255
    /// (0-254, since 255 is used as `NULL` to indicate non-existence).
    indices: [u8; N],
    entries: Vec<(Byte, Value)>,
}

impl<const N: usize, Value> ByteArrayMap<N, Value> {
    /// SAFETY: `N` must be <= 255.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            indices: [NULL; N],
            entries: Vec::with_capacity(N / 4),
        }
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.indices.fill(NULL);
        self.entries.clear();
    }

    /// SAFETY: the `byte` must be in the range of `0..N`.
    #[inline(always)]
    pub(crate) unsafe fn get(&self, byte: Byte) -> Option<&Value> {
        match unsafe { *self.indices.get_unchecked(byte as usize) } {
            NULL => None,
            index => Some(unsafe { &self.entries.get_unchecked(index as usize).1 }),
        }
    }
    /// SAFETY: the `byte` must be in the range of `0..N`.
    #[inline(always)]
    pub(crate) unsafe fn get_mut(&mut self, byte: Byte) -> Option<&mut Value> {
        match unsafe { *self.indices.get_unchecked(byte as usize) } {
            NULL => None,
            index => Some(unsafe { &mut self.entries.get_unchecked_mut(index as usize).1 }),
        }
    }

    /// SAFETY: the `byte` must be in the range of `0..N`.
    #[inline(always)]
    pub(crate) unsafe fn delete(&mut self, byte: Byte) {
        match std::mem::replace(
            unsafe { self.indices.get_unchecked_mut(byte as usize) },
            NULL,
        ) {
            NULL => (),
            prev_index => {
                let prev_index = prev_index as usize;
                self.entries.swap_remove(prev_index);
                if prev_index == self.entries.len() {
                    // removed the last element; do nothing
                } else {
                    // the last entry is now moved to `prev_index`; update its index
                    let moved_byte = unsafe { self.entries.get_unchecked(prev_index).0 };
                    unsafe {
                        *self.indices.get_unchecked_mut(moved_byte as usize) = prev_index as u8
                    };
                }
            }
        }
    }

    /// SAFETY: the `byte` must be in the range of `0..N`.
    #[inline(always)]
    pub(crate) unsafe fn insert(&mut self, byte: Byte, value: Value) {
        let index_mut = unsafe { self.indices.get_unchecked_mut(byte as usize) };
        match *index_mut {
            NULL => {
                *index_mut = self.entries.len() as u8;
                self.entries.push((byte, value));
            }
            index => {
                unsafe { self.entries.get_unchecked_mut(index as usize).1 = value };
            }
        }
    }
    /// SAFETY:
    ///
    /// 1. the `byte` must be in the range of `0..N`.
    /// 2. the `byte` must not already exist in the map.
    #[inline(always)]
    pub(crate) unsafe fn insert_new(&mut self, byte: Byte, value: Value) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                unsafe { *self.indices.get_unchecked(byte as usize) },
                NULL,
                "ByteArrayMap::insert_new: the byte `{byte}` already exists in the map"
            );
        }
        unsafe { *self.indices.get_unchecked_mut(byte as usize) = self.entries.len() as u8 };
        self.entries.push((byte, value));
    }

    #[inline(always)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(Byte, Value)> {
        self.entries.iter()
    }
}

const _: () = {
    impl<const N: usize, Value> IntoIterator for ByteArrayMap<N, Value> {
        type Item = (Byte, Value);
        type IntoIter = std::vec::IntoIter<(Byte, Value)>;
        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.entries.into_iter()
        }
    }

    impl<const N: usize, Value: PartialEq> PartialEq for ByteArrayMap<N, Value> {
        fn eq(&self, other: &Self) -> bool {
            for i in 0..N as u8 {
                if unsafe { self.get(i) } != unsafe { other.get(i) } {
                    return false;
                }
            }
            true
        }
    }

    impl<const N: usize, Value: Clone> Clone for ByteArrayMap<N, Value> {
        fn clone(&self) -> Self {
            Self {
                indices: self.indices,
                entries: self.entries.clone(),
            }
        }
    }
};

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_index_map_iter_simple() {
        let mut map = ByteArrayMap::<8, &'static str>::new();

        unsafe { map.insert(0, "a") };
        unsafe { map.insert(1, "b") };
        unsafe { map.insert(2, "c") };
        unsafe { map.insert(3, "d") };

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (1, "b"), (2, "c"), (3, "d")])
        );
    }

    #[test]
    fn test_index_map_iter_with_delete() {
        let mut map = ByteArrayMap::<8, &'static str>::new();

        unsafe { map.insert(0, "a") };
        unsafe { map.insert(1, "b") };
        unsafe { map.insert(2, "c") };
        unsafe { map.insert(3, "d") };

        unsafe { map.delete(1) };
        unsafe { map.delete(3) };

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (2, "c")])
        );
    }

    #[test]
    fn test_index_map_get_with_delete_and_other_set() {
        let mut map = ByteArrayMap::<8, &'static str>::new();

        unsafe { map.insert(0, "a") };
        unsafe { map.insert(1, "b") };
        unsafe { map.insert(2, "c") };
        unsafe { map.insert(3, "d") };

        unsafe { map.delete(1) };
        unsafe { map.delete(3) };

        unsafe { map.insert(4, "e") };
        unsafe { map.insert(5, "f") };

        assert_eq!(unsafe { map.get(0) }, Some(&"a"));
        assert_eq!(unsafe { map.get(1) }, None);
        assert_eq!(unsafe { map.get(2) }, Some(&"c"));
        assert_eq!(unsafe { map.get(3) }, None);
        assert_eq!(unsafe { map.get(4) }, Some(&"e"));
        assert_eq!(unsafe { map.get(5) }, Some(&"f"));
    }

    #[test]
    fn test_index_map_get_with_delete_and_overset() {
        let mut map = ByteArrayMap::<8, &'static str>::new();

        unsafe { map.insert(0, "a") };
        unsafe { map.insert(1, "b") };
        unsafe { map.insert(2, "c") };
        unsafe { map.insert(3, "d") };

        unsafe { map.delete(1) };
        unsafe { map.delete(3) };

        unsafe { map.insert(1, "e") };
        unsafe { map.insert(3, "f") };

        assert_eq!(unsafe { map.get(0) }, Some(&"a"));
        assert_eq!(unsafe { map.get(1) }, Some(&"e"));
        assert_eq!(unsafe { map.get(2) }, Some(&"c"));
        assert_eq!(unsafe { map.get(3) }, Some(&"f"));
    }

    #[test]
    fn test_index_map_iter_with_delete_and_overset() {
        let mut map = ByteArrayMap::<8, &'static str>::new();

        unsafe { map.insert(0, "a") };
        unsafe { map.insert(1, "b") };
        unsafe { map.insert(2, "c") };
        unsafe { map.insert(3, "d") };

        unsafe { map.delete(1) };
        unsafe { map.delete(3) };

        unsafe { map.insert(1, "e") };
        unsafe { map.insert(3, "f") };

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (2, "c"), (1, "e"), (3, "f")])
        );
    }
}
