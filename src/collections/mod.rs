use std::collections::{HashMap, HashSet};

use hash::*;

use ecs::Entity;

pub type EntityMap<T> = HashMap<Entity, T, FnvHashState>;
pub type EntitySet = HashSet<Entity, FnvHashState>;
