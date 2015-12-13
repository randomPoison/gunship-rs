#![feature(core, core_intrinsics, raw, drain, unboxed_closures, hashmap_hasher, augmented_assignments)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_audio as bs_audio;
extern crate polygon;
extern crate polygon_math as math;
extern crate hash;

pub mod stopwatch {
    extern crate stopwatch;

    #[cfg(feature="timing")]
    pub use self::stopwatch::{Collector, Stopwatch};

    #[cfg(not(feature="timing"))]
    pub use self::stopwatch::null::{Collector, Stopwatch};
}

pub mod collections;
pub mod component;
pub mod debug_draw;
pub mod engine;
pub mod ecs;
pub mod input;
pub mod resource;
pub mod scene;

mod wav;

pub use math::*;
pub use self::engine::Engine;
pub use self::scene::Scene;
pub use self::input::{Input, ScanCode};
pub use self::resource::ResourceManager;
pub use self::ecs::{Entity, EntityManager, System, ComponentManager};
pub use self::component::transform::{TransformManager, Transform};
pub use self::component::camera::{CameraManager, Camera};
pub use self::component::mesh::{MeshManager, Mesh};
pub use self::component::light::{LightManager, Light, PointLight};
pub use self::component::audio::{AudioSourceManager, AudioSource};
pub use self::component::alarm::{AlarmId, AlarmManager};
pub use self::component::collider::{ColliderManager, Collider};
pub use self::component::singleton_component_manager::SingletonComponentManager;
pub use self::component::struct_component_manager::StructComponentManager;

// TODO: These are only needed for hotloading support.
pub use self::engine::{engine_init, engine_update_and_render};
