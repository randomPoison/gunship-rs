pub mod transform;
pub mod camera;
pub mod mesh;
pub mod light;
pub mod audio;
pub mod alarm;
pub mod struct_component_manager;
pub mod collider;

pub use self::struct_component_manager::StructComponentManager;
pub use self::transform::{Transform, TransformManager, TransformUpdateSystem};
pub use self::camera::{Camera, CameraManager};
pub use self::mesh::{Mesh, MeshManager};
pub use self::light::{Light, LightManager, LightUpdateSystem};
pub use self::audio::{AudioSource, AudioSourceManager, AudioSystem};
pub use self::alarm::{AlarmID, AlarmManager, AlarmSystem};
pub use self::collider::{Collider, ColliderManager, CollisionSystem, bounding_volume, grid_collision};

use std::collections::{HashMap, HashSet};
use std::collections::hash_state::DefaultState;

use fnv::FnvHasher;

use ecs::Entity;

pub type EntityMap<T> = HashMap<Entity, T, DefaultState<FnvHasher>>;
pub type EntitySet = HashSet<Entity, DefaultState<FnvHasher>>;
