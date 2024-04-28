use std::{collections::HashMap, hash::{BuildHasherDefault, Hasher}, ops::BitXor};
use ohkami_lib::{CowSlice, Slice};


pub type HeaderHashMap = HashMap<Slice, CowSlice, BuildHasherDefault<HeaderHasher>>;


#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: usize = 0x517cc1b727220a95;

#[inline(always)]
fn take_first_chunk<'s, const N: usize>(slice: &mut &'s [u8]) -> Option<&'s [u8; N]> {
    let (first, tail) = slice.split_first_chunk()?;
    *slice = tail;
    Some(first)
}

#[inline(always)]
const fn ignore_case(byte: u8) -> u8 {
    const CASE_DIFF: u8 = b'a' - b'A';
    match byte {
        b'A'..=b'Z' => byte + CASE_DIFF,
        _           => byte,
    }
}

#[derive(Clone)]
pub struct HeaderHasher {
    hash: usize,
}

impl Default for HeaderHasher {
    #[inline(always)]
    fn default() -> Self {
        Self { hash: 0 }
    }
}

impl HeaderHasher {
    #[inline(always)]
    fn add(&mut self, word: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(word).wrapping_mul(K);
    }
}

impl Hasher for HeaderHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        let mut state = self.clone();

        while let Some(&[a, b, c, d, e, f, g, h]) = take_first_chunk(&mut bytes) {
            state.add(usize::from_ne_bytes([
                ignore_case(a),
                ignore_case(b),
                ignore_case(c),
                ignore_case(d),
                ignore_case(e),
                ignore_case(f),
                ignore_case(g),
                ignore_case(h),
            ]));
        }
        if let Some(&[a, b, c, d]) = take_first_chunk(&mut bytes) {
            state.add(u32::from_ne_bytes([
                ignore_case(a),
                ignore_case(b),
                ignore_case(c),
                ignore_case(d),
            ]) as usize);
        }
        if let Some(&[a, b]) = take_first_chunk(&mut bytes) {
            state.add(u16::from_ne_bytes([
                ignore_case(a),
                ignore_case(b),
            ]) as usize);
        }
        if let Some(&[a]) = take_first_chunk(&mut bytes) {
            state.add(ignore_case(a) as usize);
        }

        *self = state;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.hash as _
    }
}
