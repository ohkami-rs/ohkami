#![feature(test)]
extern crate test;

mod targets {
    const HASH_SIZE: usize = 1000000009;
    const PRIME:     usize = 29;
    const fn alphabet_index(alphabet: &u8) -> usize {
        (*alphabet - b'a') as usize
    }
    pub fn polynomial_rolling_hash(key: &str) -> usize {
        key.as_bytes()
            .into_iter()
            .rfold(0, |hash, byte|
                (hash * PRIME + alphabet_index(byte)) % HASH_SIZE
            )
    }

    pub fn fnv_1_hash_32bit(key: &str) -> u32 {
        key.as_bytes()
            .into_iter()
            .fold(2166136261, |hash, byte|
                (hash * 16777619) ^ (*byte as u32)
            )
    }
}

const CASE: &'static[&'static str] = &[
    "abc",
    "abcde",
    "abcdefg",
    "abcdefgh",
    "abcdefghijk",
    "abcdefghijklm",
    "abcdefghijklmnopq",
    "abcdefghijklmnopqr",
    "abcdefghijklmnopqrstuvwx",
    "abcdefghijklmnopqrstuvwxyz",
    "ab",
    "abcd",
    "abcdef",
    "abcdefg",
    "abcdefghij",
    "abcdefghijkl",
    "abcdefghijklmnop",
    "abcdefghijklmnopq",
    "abcdefghijklmnopqrstuvw",
    "abcdefghijklmnopqrstuvwxy",
    "a",
    "abc",
    "abcde",
    "abcdef",
    "abcdefghi",
    "abcdefghijk",
    "abcdefghijklmno",
    "abcdefghijklmnop",
    "abcdefghijklmnopqrstuv",
    "abcdefghijklmnopqrstuvwx",
    "aa",
    "aaa",
    "aaaa",
];

#[bench]
fn polynomial_rolling_hash(b: &mut test::Bencher) {
    b.iter(|| {
        let map = CASE
            .into_iter()
            .map(|word| targets::polynomial_rolling_hash(&word));
    })
}

#[bench]
fn fnv_1_hash_32bit(b: &mut test::Bencher) {
    b.iter(|| {
        let map = CASE
            .into_iter()
            .map(|word| targets::fnv_1_hash_32bit(&word));
    })
}