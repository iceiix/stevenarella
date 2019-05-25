

use std::hash::Hasher;

pub struct FNVHash(u64);

impl Hasher for FNVHash {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.wrapping_mul(0x100000001b3);
            self.0 ^= *b as u64
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

impl Default for FNVHash {
    fn default() -> Self {
        FNVHash(0xcbf29ce484222325)
    }
}
