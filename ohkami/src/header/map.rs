pub(crate) struct IndexMap<const N: usize, Value> {
    index:  [u8; N],
    values: Vec<(usize, Value)>,
}

/// Key-Value map mainly used to store custom headers.
/// 
/// Usually, a web app handles 0 ~ 4 custom headers, and so
/// simple `Vec<(K, V)>` is efficient than `HashMap<K, V>`
/// to store/iterate/search.
pub(crate) struct TupleMap<K: PartialEq, V>(
    Vec<(K, V)>
);

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
        match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(&self.values.get_unchecked(index as usize).1)
        }
    }
    #[inline(always)]
    pub(crate) unsafe fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(&mut self.values.get_unchecked_mut(index as usize).1)
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn delete(&mut self, index: usize) {
        *self.index.get_unchecked_mut(index) = Self::NULL
    }

    #[inline(always)]
    pub(crate) unsafe fn set(&mut self, index: usize, value: Value) {
        *self.index.get_unchecked_mut(index) = self.values.len() as u8;
        self.values.push((index, value));
    }

    #[inline(always)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(usize, Value)> {
        self.values.iter()
            .filter(|(i, _)| *unsafe {self.index.get_unchecked(*i)} != Self::NULL)
    }
}

impl<K: PartialEq, V> TupleMap<K, V> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
    pub(crate) fn from_iter<const N: usize>(iter: [(K, V); N]) -> Self {
        Self(Vec::from(iter))
    }

    #[inline]
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        for (k, v) in &self.0 {
            if key == k {return Some(v)}
        }; None
    }
    #[inline]
    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        for (k, v) in &mut self.0 {
            if key == k {return Some(v)}
        }; None
    }

    #[inline]
    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        for (k, v) in &mut self.0 {
            if &key == k {return Some(std::mem::replace(v, value))}
        }; {self.0.push((key, value)); None}
    }

    #[inline]
    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        for i in 0..self.0.len() {
            if &key == &unsafe {self.0.get_unchecked(i)}.0 {
                return Some(self.0.swap_remove(i).1)
            }
        }; None
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
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

    impl<K: PartialEq, V: PartialEq> PartialEq for TupleMap<K, V> {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl<K: Clone + PartialEq, V: Clone> Clone for TupleMap<K, V> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
};
