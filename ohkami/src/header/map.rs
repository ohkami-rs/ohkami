pub(crate) struct IndexMap<const N: usize, Value> {
    index:  [u8; N],
    values: Vec<(usize, Value)>,
}

impl<const N: usize, Value> IndexMap<N, Value> {
    const NULL: u8 = u8::MAX;

    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            index:  [Self::NULL; N],
            values: Vec::with_capacity(N / 4)
        }
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn clear(&mut self) {
        for idx in &mut self.index {*idx = Self::NULL}
        self.values.clear();
    }

    #[inline(always)]
    pub(crate) unsafe fn get(&self, index: usize) -> Option<&Value> {
        unsafe {match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(&self.values.get_unchecked(index as usize).1)
        }}
    }
    #[inline(always)]
    pub(crate) unsafe fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        unsafe {match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(&mut self.values.get_unchecked_mut(index as usize).1)
        }}
    }

    #[inline(always)]
    pub(crate) unsafe fn delete(&mut self, index: usize) {
        *unsafe {self.index.get_unchecked_mut(index)} = Self::NULL;
    }

    #[inline(always)]
    pub(crate) unsafe fn set(&mut self, index: usize, value: Value) {
        *unsafe {self.index.get_unchecked_mut(index)} = self.values.len() as u8;
        self.values.push((index, value));
    }

    #[inline(always)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &Value)> {
        self.values.iter()
            .enumerate()
            .filter_map(|(pos, (index, value))| (
                // `!= Self::NULL` can't correctly handle *over-set after delete*,
                // we MUST check the held index to be equal to the current position
                *unsafe {self.index.get_unchecked(*index)} == pos as u8
            ).then_some((*index, value)))
    }

    #[inline(always)]
    pub(crate) fn into_iter(self) -> impl Iterator<Item = (usize, Value)> {
        self.values.into_iter()
            .enumerate()
            .filter_map(move |(pos, (index, value))| (
                // `!= Self::NULL` can't correctly handle *over-set after delete*,
                // we MUST check the held index to be equal to the current position
                *unsafe {self.index.get_unchecked(index)} == pos as u8
            ).then_some((index, value)))
    }
}

const _: () = {
    impl<const N: usize, Value: PartialEq> PartialEq for IndexMap<N, Value> {
        fn eq(&self, other: &Self) -> bool {
            for i in 0..N {
                if unsafe {self.get(i)} != unsafe {other.get(i)} {
                    return false
                }
            }; true
        }
    }

    impl<const N: usize, Value: Clone> Clone for IndexMap<N, Value> {
        fn clone(&self) -> Self {
            Self {
                index:  self.index.clone(),
                values: self.values.clone(),
            }
        }
    }
};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_index_map_iter_simple() {
        let mut map = IndexMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![(0, "a"), (1, "b"), (2, "c"), (3, "d")]
        );
    }

    #[test]
    fn test_index_map_iter_with_delete() {
        let mut map = IndexMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![(0, "a"), (2, "c")]
        );
    }

    #[test]
    fn test_index_map_iter_with_delete_and_other_set() {
        let mut map = IndexMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        unsafe {map.set(4, "e")};
        unsafe {map.set(5, "f")};

        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![(0, "a"), (2, "c"), (4, "e"), (5, "f")]
        );
    }

    #[test]
    fn test_index_map_iter_with_delete_and_overset() {
        let mut map = IndexMap::<8, &'static str>::new();
    
        unsafe {map.set(0, "a")};
        unsafe {map.set(1, "b")};
        unsafe {map.set(2, "c")};
        unsafe {map.set(3, "d")};

        unsafe {map.delete(1)};
        unsafe {map.delete(3)};

        unsafe {map.set(1, "e")};
        unsafe {map.set(3, "f")};

        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![(0, "a"), (2, "c"), (1, "e"), (3, "f")]
        );
    }
}
