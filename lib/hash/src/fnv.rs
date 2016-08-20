use std::hash::{BuildHasher, Hasher};

///! This was taken directly from https://github.com/servo/rust-fnv. I needed to make the hasher
///! clonable, and this was faster than actually cloning the repo and making PR. At some point I
///! should probably request the change be added to the main repo.

#[derive(Debug, Clone, Copy, Default)]
pub struct FnvHashState;

impl BuildHasher for FnvHashState {
    type Hasher = FnvHasher;

    fn build_hasher(&self) -> FnvHasher {
        FnvHasher::default()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FnvHasher(u64);

impl Default for FnvHasher {
    #[inline]
    fn default() -> FnvHasher { FnvHasher(0xcbf29ce484222325) }
}

impl Hasher for FnvHasher {
    #[inline]
    fn finish(&self) -> u64 { self.0 }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let FnvHasher(mut hash) = *self;
        for byte in bytes.iter() {
            hash = hash ^ (*byte as u64);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        *self = FnvHasher(hash);
    }
}
