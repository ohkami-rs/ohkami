use crate::{
    result::{Result, ElseResponseWithErr},
    prelude::Response,
};

const BIG_PRIME:       usize = 124901413;
const HASH_TABLE_SIZE: usize = 97;

fn hash(key: &str) -> usize {
    key.as_bytes().into_iter().fold(0, |current_hash, ch|
        (current_hash * BIG_PRIME + *ch as usize) % HASH_TABLE_SIZE
    )
}

pub(crate) struct StringHashMap(
    [Option<String>; HASH_TABLE_SIZE]
); impl StringHashMap {
    pub fn new() -> Result<Self> {
        Ok(Self(
            TryInto::<[Option<String>; HASH_TABLE_SIZE]>::try_into(
                std::vec::from_elem(None, HASH_TABLE_SIZE)
            )._else(|_| Response::InternalServerError("Failed in type casting"))?
        ))
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.as_ref()
            .get(hash(key))?
            .as_ref()
            .map(|string| &**string)
    }
    pub fn insert(&mut self, key: &str, value: String) {
        self.0[hash(key)] = Some(value)
    }
}
