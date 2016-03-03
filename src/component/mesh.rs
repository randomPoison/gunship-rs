use ecs::*;
use engine::*;
use polygon::gl_render::{GLMeshData, ShaderProgram};
use super::struct_component_manager::*;


#[derive(Debug, Clone)]
pub struct Mesh {
    entity:  Entity,
    gl_mesh: GLMeshData,
    shader:  ShaderProgram,
}

impl Mesh {
    pub fn gl_mesh(&self) -> &GLMeshData {
        &self.gl_mesh
    }

    pub fn set_shader(&self, uri: &'static str) {
        let shader =
            Engine::resource_manager()
            .get_shader(uri)
            .unwrap(); // TODO: Provide better panic message (or maybe DON'T PANIC!?).

        Engine::scene()
        .manager_for::<Mesh>()
        .send_message(self.entity, MeshMessage::SetShader(shader));
    }

    pub fn shader(&self) -> &ShaderProgram {
        &self.shader
    }
}

impl Component for Mesh {
    type Manager = MeshManager;
    type Message = MeshMessage;
}

#[derive(Debug, Clone)]
pub struct MeshManager(StructComponentManager<Mesh>);

impl MeshManager {
    pub fn assign(&self, entity: Entity, path_text: &str) -> &Mesh {
        let mesh =
            Engine::resource_manager()
            .get_gpu_mesh(path_text)
            .ok_or_else(|| format!("ERROR: Unable to assign mesh with uri {}", path_text))
            .unwrap(); // TODO: Provide better panic message (it's okay to panic here though, indicates a bug in game code).
        self.give_mesh(entity, mesh)
    }

    pub fn give_mesh(&self, entity: Entity, mesh: GLMeshData) -> &Mesh {
        let shader =
            Engine::resource_manager()
            .get_shader("shaders/forward_phong.glsl")
            .unwrap(); // TODO: Provide better panic message (or maybe DON'T PANIC!?).
        self.0.assign(entity, Mesh {
            entity:  entity,
            gl_mesh: mesh,
            shader:  shader,
        })
    }

    pub fn iter(&self) -> Iter<Mesh> {
        self.0.iter()
    }

    fn send_message(&self, entity: Entity, message: MeshMessage) {
        self.0.send_message(entity, message);
    }
}

impl ComponentManagerBase for MeshManager {
    fn update(&mut self) {
        self.0.process_messages();
    }
}

impl ComponentManager for MeshManager {
    type Component = Mesh;

    fn register(builder: &mut EngineBuilder) {
        let mesh_manager =
            MeshManager(StructComponentManager::new());
        builder.register_manager(mesh_manager);
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.0.get(entity)
    }

    fn destroy(&self, entity: Entity) {
        self.0.destroy(entity);
    }
}

derive_Singleton!(MeshManager);

#[derive(Debug, Clone)]
pub enum MeshMessage {
    SetShader(ShaderProgram),
}

impl Message for MeshMessage {
    type Target = Mesh;

    fn apply(self, target: &mut Mesh) {
        match self {
            MeshMessage::SetShader(shader) => {
                target.shader = shader;
            },
        }
    }
}
