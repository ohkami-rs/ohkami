// const HASH_SIZE: usize = 20000063;
// const PRIME:     usize = 29;
// const fn alphabet_index(alphabet: &u8) -> usize {
//     (*alphabet - b'a') as usize
// }
// fn polynomial_rolling_hash(key: &str) -> usize {
//     key.as_bytes()
//         .into_iter()
//         .rfold(0, |hash, byte|
//             (hash * PRIME + alphabet_index(byte)) % HASH_SIZE
//         )
// }

// fn fnv_1_hash_32bit(key: &str) -> u32 {
//     key.as_bytes()
//         .into_iter()
//         .fold(2166136261, |hash, byte|
//             (hash * 16777619) ^ (*byte as u32)
//         )
// }

const BIG_PRIME:  usize = 1212121;
pub(super) const TABLE_SIZE: usize = 2357;

pub(super) fn linear_congruential_hash(key: &str) -> usize {
    key.as_bytes()
        .into_iter()
        .fold(0, |hash, byte|
            (hash * BIG_PRIME + *byte as usize) % TABLE_SIZE
        )
}
