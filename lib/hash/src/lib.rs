#![feature(hashmap_hasher)]
extern crate twox_hash;

pub mod fnv;
// pub mod xxhash;

pub use fnv::{FnvHashState, FnvHasher};
pub use twox_hash::*;
