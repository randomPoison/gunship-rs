use ecs::*;
use engine::*;
use polygon::gl_render::{GLMeshData, ShaderProgram};
use super::DefaultMessage;
use super::struct_component_manager::*;


#[derive(Debug, Clone)]
pub struct Mesh {
    pub gl_mesh: GLMeshData,
    pub shader: ShaderProgram,
}

impl Component for Mesh {
    type Manager = MeshManager;
    type Message = DefaultMessage<Mesh>;
}

#[derive(Debug, Clone)]
pub struct MeshManager(StructComponentManager<Mesh>);

impl MeshManager {
    pub fn new() -> MeshManager {
        MeshManager(StructComponentManager::new())
    }

    pub fn assign(&mut self, entity: Entity, path_text: &str) -> &Mesh {
        let mesh =
            Engine::resource_manager()
            .get_gpu_mesh(path_text)
            .ok_or_else(|| format!("ERROR: Unable to assign mesh with uri {}", path_text))
            .unwrap(); // TODO: Provide better panic message (it's okay to panic here though, indicates a bug in game code).
        self.give_mesh(entity, mesh)
    }

    pub fn give_mesh(&mut self, entity: Entity, mesh: GLMeshData) -> &Mesh {
        let shader =
            Engine::resource_manager()
            .get_shader("shaders/forward_phong.glsl")
            .unwrap(); // TODO: Provide better panic message (or maybe DON'T PANIC!?).
        self.0.assign(entity, Mesh {
            gl_mesh: mesh,
            shader: shader,
        })
    }

    pub fn iter(&self) -> Iter<Mesh> {
        self.0.iter()
    }
}

impl ComponentManagerBase for MeshManager {}

impl ComponentManager for MeshManager {
    type Component = Mesh;

    fn register(builder: &mut EngineBuilder) {
        let mesh_manager = MeshManager::new();
        builder.register_manager(mesh_manager);
    }

    fn destroy(&self, entity: Entity) {
        self.0.destroy(entity);
    }
}

derive_Singleton!(MeshManager);
