use ohkami_lib::{CowSlice, Slice};
use rustc_hash::FxHashMap;


pub struct FxMap(FxHashMap<
    Slice, CowSlice
>);

impl FxMap {
    pub fn new() -> Self {
        Self(FxHashMap::default())
    }

    #[inline(always)]
    pub fn insert(&mut self, key: Slice, value: CowSlice) {
        self.0.insert(key, value);
    }

}
