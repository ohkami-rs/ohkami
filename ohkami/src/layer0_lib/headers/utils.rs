use std::hash::{Hash, Hasher};
use super::hash::{FNVHasher};
use super::headers::{Size, HashValue, MAX_SIZE};


#[inline] pub(super) fn usable_capacity(capacity: usize) -> usize {
    capacity - capacity / 4
}

#[inline] pub(super) fn to_raw_capacity(n: usize) -> usize {
    n.checked_add(n / 3).unwrap_or_else(|| panic!("Requested too large capacity `{n}`: overflow while converting to raw capacity"))
}

#[inline] pub(super) fn desired_pos(mask: Size, hash: HashValue) -> usize {
    (hash & mask) as _
}

#[inline] pub(super) fn probe_distance(mask: Size, hash: HashValue, current: usize) -> usize {
    current.wrapping_sub(desired_pos(mask, hash)) & mask as usize
}

pub(super) fn hash_elem<Key: Hash>(key: &Key) -> HashValue {
    let mut h = FNVHasher::default();
    key.hash(&mut h);
    (h.finish() & (MAX_SIZE as u64) - 1) as HashValue
}
