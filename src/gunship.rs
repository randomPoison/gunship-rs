#![feature(core_intrinsics, raw, drain)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_audio as bs_audio;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

pub mod engine;
pub mod scene;
pub mod input;
pub mod resource;
pub mod ecs;
pub mod component;

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
pub use self::component::alarm::{AlarmID, AlarmManager};
pub use self::component::struct_component_manager::StructComponentManager;

// TODO: These are only needed for hotloading support.
pub use self::engine::{engine_init, engine_update_and_render};
