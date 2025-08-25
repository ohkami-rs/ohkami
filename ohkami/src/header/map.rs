type Byte = u8;

const NULL: Byte = Byte::MAX;

pub(crate) struct ByteArrayMap<const N: usize, Value> {
    indices: [u8; N], // using `u8` to save memory space, max capacity is 255
    entries: Vec<(Byte, Value)>,
}

impl<const N: usize, Value> ByteArrayMap<N, Value> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            indices: [NULL; N],
            entries: Vec::with_capacity(N / 4)
        }
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.indices.fill(NULL);
        self.entries.clear();
    }

    #[inline(always)]
    pub(crate) unsafe fn get(&self, byte: Byte) -> Option<&Value> {
        match unsafe {*self.indices.get_unchecked(byte as usize)} {
            NULL  => None,
            index => Some(unsafe {&self.entries.get_unchecked(index as usize).1})
        }
    }
    #[inline(always)]
    pub(crate) unsafe fn get_mut(&mut self, byte: Byte) -> Option<&mut Value> {
        match unsafe {*self.indices.get_unchecked(byte as usize)} {
            NULL  => None,
            index => Some(unsafe {&mut self.entries.get_unchecked_mut(index as usize).1})
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn delete(&mut self, byte: Byte) {
        let prev_index = std::mem::replace(
            unsafe {self.indices.get_unchecked_mut(byte as usize)},
            NULL
        );
        if prev_index != NULL {
            let prev_index = prev_index as usize;
            self.entries.swap_remove(prev_index);
            if prev_index == self.entries.len() {
                // removed the last element; do nothing
            } else {
                // the last entry is now moved to `prev_index`; update its index
                let moved_byte = unsafe {self.entries.get_unchecked(prev_index).0};
                unsafe {*self.indices.get_unchecked_mut(moved_byte as usize) = prev_index as u8};
            }
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn set(&mut self, byte: Byte, value: Value) {
        let index_mut = unsafe {self.indices.get_unchecked_mut(byte as usize)};
        match *index_mut {
            NULL => {
                *index_mut = self.entries.len() as u8;
                self.entries.push((byte, value));
            }
            index => {
                unsafe {self.entries.get_unchecked_mut(index as usize).1 = value};
            }
        }
    }

    #[inline(always)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(Byte, Value)> {
        self.entries.iter()
    }
    #[inline(always)]
    pub(crate) fn into_iter(self) -> impl Iterator<Item = (Byte, Value)> {
        self.entries.into_iter()
    }
}

const _: () = {
    impl<const N: usize, Value: PartialEq> PartialEq for ByteArrayMap<N, Value> {
        fn eq(&self, other: &Self) -> bool {
            for i in 0..N as u8 {
                if unsafe {self.get(i)} != unsafe {other.get(i)} {
                    return false
                }
            }; true
        }
    }

    impl<const N: usize, Value: Clone> Clone for ByteArrayMap<N, Value> {
        fn clone(&self) -> Self {
            Self {
                indices: self.indices.clone(),
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
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (1, "b"), (2, "c"), (3, "d")])
        );
    }

    #[test]
    fn test_index_map_iter_with_delete() {
        let mut map = ByteArrayMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (2, "c")])
        );
    }

    #[test]
    fn test_index_map_get_with_delete_and_other_set() {
        let mut map = ByteArrayMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        unsafe {map.set(4, "e")};
        unsafe {map.set(5, "f")};

        assert_eq!(unsafe {map.get(0)}, Some(&"a"));
        assert_eq!(unsafe {map.get(1)}, None);
        assert_eq!(unsafe {map.get(2)}, Some(&"c"));
        assert_eq!(unsafe {map.get(3)}, None);
        assert_eq!(unsafe {map.get(4)}, Some(&"e"));
        assert_eq!(unsafe {map.get(5)}, Some(&"f"));
    }

    #[test]
    fn test_index_map_get_with_delete_and_overset() {
        let mut map = ByteArrayMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        unsafe {map.set(1, "e")};
        unsafe {map.set(3, "f")};

        assert_eq!(unsafe {map.get(0)}, Some(&"a"));
        assert_eq!(unsafe {map.get(1)}, Some(&"e"));
        assert_eq!(unsafe {map.get(2)}, Some(&"c"));
        assert_eq!(unsafe {map.get(3)}, Some(&"f"));
    }

    #[test]
    fn test_index_map_iter_with_delete_and_overset() {
        let mut map = ByteArrayMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        unsafe {map.set(1, "e")};
        unsafe {map.set(3, "f")};

        assert_eq!(
            map.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter([(0, "a"), (2, "c"), (1, "e"), (3, "f")])
        );
    }
}
