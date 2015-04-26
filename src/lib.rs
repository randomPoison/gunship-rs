#![feature(collections)]

extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

pub mod engine;
pub mod scene;
pub mod input;
pub mod resource;
pub mod ecs;
pub mod component;

pub use math::*;
pub use self::engine::Engine;
pub use self::scene::Scene;
pub use self::input::{Input, ScanCode};
pub use self::resource::ResourceManager;
pub use self::ecs::{EntityManager, System, ComponentManager};
pub use self::component::transform::{TransformManager, Transform};
pub use self::component::camera::{CameraManager, Camera};
pub use self::component::mesh::{MeshManager, Mesh};
pub use self::component::struct_component_manager::StructComponentManager;
