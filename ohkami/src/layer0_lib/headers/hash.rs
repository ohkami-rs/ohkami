pub struct FNVHasher(u64);
const INITIAL_STATE: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0100_0000_01b3;

impl Default for FNVHasher {
    #[inline] fn default() -> Self {
        FNVHasher(INITIAL_STATE)
    }
}

impl std::hash::Hasher for FNVHasher {
    #[inline] fn finish(&self) -> u64 {
        self.0
    }
    #[inline] fn write(&mut self, bytes: &[u8]) {
        let Self(mut h) = *self;
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(PRIME);
        }
        *self = Self(h);
    }
}
