extern crate bootstrap_rs as bootstrap;
extern crate parse_collada as collada;
extern crate polygon_rs as polygon;
extern crate polygon_math as math;

mod components;
mod entity;
mod input;

use std::f32::consts::PI;

use bootstrap::window::Window;
use bootstrap::window::Message::*;
use bootstrap::input::ScanCode;

use math::point::{point};
use math::vector::{vector3};
use math::matrix::Matrix4;

use polygon::gl_render;

use entity::EntityManager;
use components::transform::{TransformManager};
use components::camera::CameraManager;
use components::mesh::MeshManager;

fn main() {
    let mut close = false;

    let instance = bootstrap::init();

    let mut window = Window::new("Rust Window", instance);

    let renderer = gl_render::init(&window);

    let mut entity_manager = EntityManager::new();
    let camera_entity = entity_manager.create();

    let mut transform_manager = TransformManager::new();
    let mut transform = transform_manager.create(camera_entity);
    transform.position = point(5.0, 0.0, 5.0);
    transform.update();

    let mut mesh_manager = MeshManager::new();
    let mesh = mesh_manager.create(camera_entity, &renderer, "meshes/gun_small.dae");

    let mut camera_manager = CameraManager::new();

    let mut camera = camera_manager.create(
        camera_entity,
        PI / 3.0,
        1.0,
        0.001,
        100.0);
    camera.position = point(5.0, 0.0, 5.0);
    camera.look_at(point(0.0, 0.0, 0.0), vector3(0.0, 1.0, 0.0));

    let mut forward = false;
    let mut backward = false;
    let mut left = false;
    let mut right = false;

    let mut last_x = 400;
    let mut last_y = 400;

    let mut rotation_x = 0.0;
    let mut rotation_y = 0.0;

    loop {
        window.handle_messages();
        loop {
            match window.next_message() {
                Some(message) => {
                    match message {
                        Activate => (),
                        Close => close = true,
                        Destroy => (),
                        Paint => (),

                        // Handle inputs.
                        KeyDown(ScanCode::W) => forward = true,
                        KeyUp(ScanCode::W) => forward = false,

                        KeyDown(ScanCode::S) => backward = true,
                        KeyUp(ScanCode::S) => backward = false,

                        KeyDown(ScanCode::D) => right = true,
                        KeyUp(ScanCode::D) => right = false,

                        KeyDown(ScanCode::A) => left = true,
                        KeyUp(ScanCode::A) => left = false,

                        MouseMove(x_coord, y_coord) => {
                            let movement_x = last_x - x_coord;
                            let movement_y = last_y - y_coord;

                            // Add mouse movement to total rotation.
                            rotation_x += (movement_y as f32) * PI * 0.001;
                            rotation_y += (movement_x as f32) * PI * 0.001;

                            // Apply a rotation to the camera based on mouse movmeent.
                            camera.rotation =
                                Matrix4::rotation(rotation_x,
                                                  rotation_y,
                                                  0.0);

                            // Save mouse coordinates.
                            last_x = x_coord;
                            last_y = y_coord;
                        }
                        _ => ()
                    }
                },
                None => break
            }
        }

        // Calculate the forward and right vectors.
        let forward_dir = -camera.rotation.z_part();
        let right_dir = camera.rotation.x_part();

        // Move camera based on input.
        if forward {
            camera.position = camera.position + forward_dir * 0.01;
        }

        if backward {
            camera.position = camera.position - forward_dir * 0.01;
        }

        if right {
            camera.position = camera.position + right_dir * 0.01;
        }

        if left {
            camera.position = camera.position - right_dir * 0.01
        }

        // mesh_transform = frame_rotation * mesh_transform;
        renderer.draw_mesh(&mesh, transform.matrix(), &camera);

        if close {
            break;
        }
    };
}
