use ecs::Entity;
use hash::*;
use std::collections::{HashMap, HashSet};

pub use self::array::Array;

pub mod array;

pub type EntityMap<T> = HashMap<Entity, T, FnvHashState>;
pub type EntitySet = HashSet<Entity, FnvHashState>;
