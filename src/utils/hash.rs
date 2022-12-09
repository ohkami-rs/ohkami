use crate::components::consts::HASH_TABLE_SIZE;

const BIG_PRIME: usize = 124901413;

pub(crate) fn hash(key: &str) -> usize {
    let mut hash = 0;
    for ch in key.as_bytes() {
        hash = (hash * BIG_PRIME + *ch as usize) % HASH_TABLE_SIZE
    }
    hash
}