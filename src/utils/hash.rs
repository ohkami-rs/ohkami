use crate::{
    result::{Result, ElseResponseWithErr},
    prelude::Response,
};

const HASH_SIZE: usize = 1000000009;
const PRIME:     usize = 29;
const fn alphabet_index(alphabet: &u8) -> usize {
    (*alphabet - b'a') as usize
}
fn polynomial_rolling_hash(key: &str) -> usize {
    key.as_bytes()
        .into_iter()
        .rfold(0, |hash, byte|
            (hash * PRIME + alphabet_index(byte)) % HASH_SIZE
        )
}

// fn fnv_1_hash_32bit(key: &str) -> u32 {
//     key.as_bytes()
//         .into_iter()
//         .fold(2166136261, |hash, byte|
//             (hash * 16777619) ^ (*byte as u32)
//         )
// }

pub(crate) struct StringHashMap(
    [Option<String>; HASH_SIZE]
); impl StringHashMap {
    pub fn new() -> Result<Self> {
        Ok(Self(
            TryInto::<[Option<String>; HASH_SIZE]>::try_into(
                std::vec::from_elem(None, HASH_SIZE)
            )._else(|_| Response::InternalServerError("Failed in type casting"))?
        ))
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.as_ref()
            .get(polynomial_rolling_hash(key))?
            .as_ref()
            .map(|string| &**string)
    }
    pub fn insert(&mut self, key: &str, value: String) {
        self.0[polynomial_rolling_hash(key)] = Some(value)
    }
}
