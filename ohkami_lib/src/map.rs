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

    pub fn append(&mut self, another: Self) {
        for (k, v) in another.into_iter() {
            self.insert(k, v);
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
    }
    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.0.into_iter()
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
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

impl<K:PartialEq, V> std::fmt::Debug for TupleMap<K, V>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|&(ref k, ref v)| (k, v)))
            .finish()
    }
}

impl<'de, K:PartialEq, V> serde::Deserialize<'de> for TupleMap<K, V>
where
    K: serde::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        return deserializer.deserialize_map(TupleMapVisitor(std::marker::PhantomData));

        /////////////////////////////////////////////////////////////////////////
        
        struct TupleMapVisitor<K, V>(std::marker::PhantomData<fn()->(K, V)>);

        impl<'de, K:PartialEq, V> serde::de::Visitor<'de> for TupleMapVisitor<K, V>
        where
            K: serde::Deserialize<'de>,
            V: serde::Deserialize<'de>,
        {
            type Value = TupleMap<K, V>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a map")
            }

            #[inline]
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut map = TupleMap::new();
                while let Some((k, v)) = access.next_entry()? {
                    map.insert(k, v);
                }
                Ok(map)
            }
        }
    }
}
