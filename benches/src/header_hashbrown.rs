use std::hash::BuildHasherDefault;
use hashbrown::HashMap;
use rustc_hash::FxHasher;
use ohkami_lib::{Slice, CowSlice};


pub struct HeaderHashBrown(HashMap<
    Slice,
    CowSlice,
    BuildHasherDefault<FxHasher>,
>);

impl HeaderHashBrown {
    pub fn new() -> Self {
        Self(HashMap::with_capacity_and_hasher(32, BuildHasherDefault::default()))
    }

    pub fn insert(&mut self, key: Slice, value: CowSlice) {
        self.0.insert(key, value);
    }
    /// SAFETY: `known_hash` is actually the hash of `key`
    pub unsafe fn insert_known(&mut self, known_hash: u64, key: Slice, value: CowSlice) {
        self.0.raw_entry_mut().from_key_hashed_nocheck(known_hash, &key)
            .insert(key, value);
    }

    pub fn remove
}
