use std::hash::{BuildHasher, Hasher};
use xxhash_rust::xxh3::Xxh3;

pub struct Xxh3Hasher {
    hasher: Xxh3,
}

impl Hasher for Xxh3Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    fn finish(&self) -> u64 {
        self.hasher.finish()
    }
}

pub struct Xxh3HasherBuilder;

impl BuildHasher for Xxh3HasherBuilder {
    type Hasher = Xxh3Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        Xxh3Hasher {
            hasher: Xxh3::new(),
        }
    }
}
