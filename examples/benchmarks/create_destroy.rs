extern crate gunship;
extern crate rand;

use std::collections::VecDeque;
use std::f32::consts::PI;

use gunship::*;

const ENTITIES_TO_CREATE: usize = 10;
const ENTITIES_TO_DESTROY: usize = 1;

fn main() {
    let mut engine = Engine::new();

    engine.register_system(CreateDestroySystem {
        entities: VecDeque::new(),
    });
    setup_scene(engine.scene_mut());

    engine.main_loop();
}

fn setup_scene(scene: &mut Scene) {
    scene.resource_manager().set_vert_shader_path("examples/shaders/forward_phong.vert.glsl");
    scene.resource_manager().set_frag_shader_path("examples/shaders/forward_phong.frag.glsl");
    scene.resource_manager().load_model("examples/meshes/cube.dae").unwrap();

    let mut transform_manager = scene.get_manager_mut::<TransformManager>();
    let mut camera_manager = scene.get_manager_mut::<CameraManager>();
    let mut light_manager = scene.get_manager_mut::<LightManager>();

    // Create camera.
    {
        let camera = scene.create_entity();
        let mut camera_transform = transform_manager.assign(camera);
        camera_transform.set_position(Point::new(0.0, 0.0, 30.0));
        camera_transform.look_at(Point::origin(), Vector3::new(0.0, 0.0, -1.0));
        camera_manager.assign(
            camera,
            Camera::new(
                PI / 3.0,
                1.0,
                0.001,
                100.0));
    }

    // Create light.
    {
        let light = scene.create_entity();
        transform_manager.assign(light);
        light_manager.assign(
            light,
            Light::Point(PointLight {
                position: Point::origin()
            }));
    }
}

#[derive(Debug, Clone)]
struct CreateDestroySystem {
    entities: VecDeque<Entity>,
}

impl System for CreateDestroySystem {
    fn update(&mut self, scene: &Scene, _delta: f32) {
        let mut transform_manager = scene.get_manager_mut::<TransformManager>();
        let mut mesh_manager = scene.get_manager_mut::<MeshManager>();

        while self.entities.len() < ENTITIES_TO_CREATE {
            let entity = scene.create_entity();

            let mut transform = transform_manager.assign(entity);
            transform.set_position(Point::new(0.0, 0.0, 0.0));
            mesh_manager.assign(entity, "cube.pCube1");

            self.entities.push_back(entity);
        }
    }
}

