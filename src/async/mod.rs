//! Provides mangement and scheduling for fiber-based tasks in the engine.
//!
//! This module acts as a singleton. This is to allow the scheduler to globally accessible, making
//! async operations usable from anywhere in the engine and game code.

pub mod camera;
pub mod collections;
pub mod engine;
pub mod mesh_renderer;
pub mod prelude;
pub mod resource;
pub mod scheduler;
pub mod time;
pub mod transform;
