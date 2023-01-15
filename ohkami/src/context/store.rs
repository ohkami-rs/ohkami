use std::collections::{HashMap, hash_map::Entry};
use fxhash::FxBuildHasher;

pub struct Store(
    HashMap<String, String, FxBuildHasher>
); impl Store {
    pub(crate) fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.0.insert(key, value)
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
    pub fn entry(&mut self, key: String) -> Entry<String, String> {
        self.0.entry(key)
    }
}


#[cfg(test)]
mod test {
    use async_std::sync::{Arc, Mutex};
    use super::Store;
    
    #[test]
    fn use_store() {
        let store = Arc::new(Mutex::new(Store::new()));
        async_std::task::block_on(async {

            store.lock().await
                .insert(format!("k1"), format!("v1"));
            assert_eq!(
                store.lock().await.get("k1"),
                Some("v1")
            );

            store.lock().await
                .entry(format!("k2"))
                .and_modify(|v| {*v = format!("modified")})
                .or_insert(format!("v2"));
            assert_eq!(
                store.lock().await.get("k2"),
                Some("v2")
            );
            store.lock().await
                .entry(format!("k2"))
                .and_modify(|v| {*v = format!("modified")})
                .or_insert(format!("v2"));
            assert_eq!(
                store.lock().await.get("k2"),
                Some("modified")
            )
        });
    }
}
