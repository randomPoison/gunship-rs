#![feature(associated_type_defaults)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fn_traits)]
#![feature(fnbox)]
#![feature(question_mark)]
#![feature(raw)]
#![feature(unboxed_closures)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_audio as bs_audio;
extern crate fiber;
extern crate hash;
extern crate polygon;
extern crate polygon_math as math;

pub mod stopwatch {
    extern crate stopwatch;

    #[cfg(feature="timing")]
    pub use self::stopwatch::{Collector, Stopwatch};

    #[cfg(not(feature="timing"))]
    pub use self::stopwatch::null::{Collector, Stopwatch};
}

#[macro_use]
pub mod macros;

pub mod async;
pub mod callback;
pub mod collections;
pub mod component;
pub mod debug_draw;
pub mod engine;
pub mod ecs;
pub mod input;
pub mod resource;
pub mod scene;
pub mod singleton;

mod wav;

pub use math::*;
pub use self::engine::{Engine, EngineBuilder};
pub use self::scene::Scene;
pub use self::input::{Input, ScanCode};
pub use self::resource::ResourceManager;
pub use self::ecs::*;
pub use self::component::transform::{TransformManager, Transform};
pub use self::component::camera::{CameraManager, Camera};
pub use self::component::{DefaultManager, DefaultMessage};
pub use self::component::mesh::{MeshManager, Mesh};
pub use self::component::light::{Light, LightManager, PointLight};
pub use self::component::audio::{AudioSourceManager, AudioSource};
pub use self::component::alarm::{AlarmId, AlarmManager};
pub use self::component::collider::{ColliderManager, Collider};
pub use self::component::singleton_component_manager::{SingletonComponentManager, SingletonComponentUpdateManager};
pub use self::polygon::geometry::mesh::MeshBuilder;
pub use self::singleton::Singleton;

// // TODO: These are only needed for hotloading support.
// pub use self::engine::{engine_init, engine_update_and_render};
