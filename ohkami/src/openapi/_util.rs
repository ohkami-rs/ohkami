use super::schema::SchemaRef;
use serde::{ser::SerializeMap, Serialize};

pub(crate) const fn is_false(bool: &bool) -> bool {
    !*bool
}

#[derive(Serialize)]
pub(crate) struct Content {
    schema: SchemaRef
}
impl<T: Into<SchemaRef>> From<T> for Content {
    fn from(schema: T) -> Self {
        Self { schema: schema.into() }
    }
}

pub(crate) struct Map<K:PartialEq, V>(Vec<(K, V)>);
impl<K:PartialEq, V> Map<K, V> {
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn from_iter(iter: impl IntoIterator<Item = (K, V)>) -> Self {
        Self(Vec::from_iter(iter))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn find(&self, key: &K) -> Option<usize> {
        self.0.iter().position(|(k, _)| k == key)
    }
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.find(key).map(|i| &self.0[i].1)
    }
    pub(crate) fn insert(&mut self, key: K, value: V) {
        match self.find(&key) {
            Some(i) => self.0[i].1 = value,
            None    => self.0.push((key, value)),
        }
    }
}
const _: () = {
    impl<K:PartialEq, V> Serialize for Map<K, V>
    where
        K:Serialize, V:Serialize
    {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut s = serializer.serialize_map(Some(self.0.len()))?;
            for (k, v) in &self.0 {
                s.serialize_entry(k, v)?;
            }
            s.end()
        }
    }

    impl<K:PartialEq, V> PartialEq for Map<K, V>
    where
        V: PartialEq
    {
        fn eq(&self, other: &Self) -> bool {
            for (k, v) in &self.0 {
                if other.get(k) != Some(v) {
                    return false
                }
            }
            true
        }
    }

    impl<K:PartialEq, V> IntoIterator for Map<K, V> {
        type Item = (K, V);
        type IntoIter = std::vec::IntoIter<(K, V)>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<K:PartialEq, V> Into<Vec<(K, V)>> for Map<K, V> {
        fn into(self) -> Vec<(K, V)> {
            self.0
        }
    }

    impl<K:PartialEq, V> Default for Map<K, V> {
        fn default() -> Self {
            Self(Vec::new())
        }
    }
};
