/// Key-Value map mainly used to store custom headers.
/// 
/// Usually, a web app handles 0 ~ 4 custom headers, and so
/// simple `Vec<(K, V)>` is efficient than `HashMap<K, V>`
/// to store/iterate/mutate.
pub struct TupleMap<K: PartialEq, V>(
    Vec<(K, V)>
);

impl<K: PartialEq, V> TupleMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn from_iter<const N: usize>(iter: [(K, V); N]) -> Self {
        Self(Vec::from(iter))
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        for (k, v) in &self.0 {
            if key == k {return Some(v)}
        }; None
    }
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        for (k, v) in &mut self.0 {
            if key == k {return Some(v)}
        }; None
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        for (k, v) in &mut self.0 {
            if &key == k {return Some(std::mem::replace(v, value))}
        }; {self.0.push((key, value)); None}
    }

    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        for i in 0..self.0.len() {
            if &key == &unsafe {self.0.get_unchecked(i)}.0 {
                return Some(self.0.swap_remove(i).1)
            }
        }; None
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
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
