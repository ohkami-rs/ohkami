use crate::{components::consts::HASH_TABLE_SIZE, result::Result};

const BIG_PRIME:       usize = 124901413;
const HASH_TABLE_SIZE: usize = 97;

fn hash(key: &str) -> usize {
    let mut hash = 0;
    for ch in key.as_bytes() {
        hash = (hash * BIG_PRIME + *ch as usize) % HASH_TABLE_SIZE
    }
    hash
}

pub(crate) struct StringHashMap(
    [Option<String>; HASH_TABLE_SIZE]
); impl StringHashMap {
    pub fn new() -> Result<Self> {
        TryInto::<[Option<String>; HASH_TABLE_SIZE]>::try_into(
            std::vec::from_elem(None, HASH_TABLE_SIZE)
        ).else_response(|_| Response::InternalServerError("Failed in type casting"))
    }
    pub fn get(&self, key: &str) -> &Option<&str> {
        self.0[hash(key)]
    }
}
