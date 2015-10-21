#![feature(hashmap_hasher)]

pub mod fnv;

pub use fnv::{FnvHashState, FnvHasher};
