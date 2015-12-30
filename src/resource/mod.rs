use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

use polygon::gl_render::{GLRender, GLMeshData, ShaderProgram};
use polygon::geometry::mesh::Mesh;

use wav::Wave;
use scene::Scene;
use ecs::Entity;
use component::{MeshManager, TransformManager};
use self::shader::*;

pub mod collada;
pub mod shader;

#[derive(Debug, Clone)]
pub struct ResourceManager {
    renderer: Rc<GLRender>,
    meshes: RefCell<HashMap<String, Mesh>>,
    gpu_meshes: RefCell<HashMap<String, GLMeshData>>,
    mesh_nodes: RefCell<HashMap<String, MeshNode>>,
    shaders: RefCell<HashMap<String, ShaderProgram>>,
    audio_clips: RefCell<HashMap<String, Rc<Wave>>>,

    resource_path: RefCell<PathBuf>,
}

impl ResourceManager {
    pub fn new(renderer: Rc<GLRender>) -> ResourceManager {
        ResourceManager {
            renderer: renderer,
            meshes: RefCell::new(HashMap::new()),
            gpu_meshes: RefCell::new(HashMap::new()),
            mesh_nodes: RefCell::new(HashMap::new()),
            shaders: RefCell::new(HashMap::new()),
            audio_clips: RefCell::new(HashMap::new()),

            resource_path: RefCell::new(PathBuf::new()),
        }
    }

    pub fn load_resource_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut full_path = self.resource_path.borrow().clone();
        full_path.push(path);
        let metadata = match fs::metadata(&full_path) {
            Err(why) => return Err(format!(
                "Unable to read metadata for {}, either it doesn't exist or the user lacks permissions, {}",
                full_path.display(),
                &why)),
            Ok(metadata) => metadata,
        };

        if !metadata.is_file() {
            return Err(format!(
                "{} could not be loaded because it is not a file",
                full_path.display()));
        }

        collada::load_resources(full_path, self).unwrap(); // TODO: Don't panic?

        Ok(())
    }

    /// Sets the path to the resources directory.
    ///
    /// # Details
    ///
    /// The resource manager is configured to look in the specified directory when loading
    /// resources such as meshes and shaders.
    pub fn set_resource_path<P: AsRef<Path>>(&self, path: P) {
        let mut resource_path = self.resource_path.borrow_mut();
        *resource_path = PathBuf::new();
        resource_path.push(path);
    }

    pub fn get_gpu_mesh(&self, uri: &str) -> Option<GLMeshData> {
        // Use cached mesh data if possible.
        self.get_cached_mesh(uri)
        .or_else(|| {
            self.gen_gpu_mesh(uri)
        })
    }

    pub fn get_audio_clip(&self, path_text: &str) -> Rc<Wave> {
        let mut audio_clips = self.audio_clips.borrow_mut();

        if !audio_clips.contains_key(path_text) {
            let wave = Wave::from_file(path_text).unwrap();
            audio_clips.insert(path_text.into(), Rc::new(wave));
        }

        audio_clips.get(path_text).unwrap().clone()
    }

    pub fn instantiate_model(&self, resource: &str, scene: &Scene) -> Result<Entity, String> {
        if resource.contains(".") {
            println!("WARNING: ResourceManager::instantiate_model() doesn't yet support fully qualified URIs, only root assets may be instantiated.");
            unimplemented!();
        }

        let mesh_nodes = self.mesh_nodes.borrow();
        let root = try!(
            mesh_nodes
            .get(resource)
            .ok_or_else(|| format!("No mesh node is identified by the uri {}", resource)));

        self.instantiate_node(scene, root)
    }

    fn instantiate_node(&self, scene: &Scene, node: &MeshNode) -> Result<Entity, String> {
        let entity = scene.create_entity();
        {
            let mut transform_manager = scene.get_manager_mut::<TransformManager>();
            transform_manager.assign(entity);

            for mesh_id in &node.mesh_ids {
                let gpu_mesh = match self.get_gpu_mesh(&*mesh_id) {
                    Some(gpu_mesh) => gpu_mesh,
                    None => {
                        println!("WARNING: Unable to load gpu mesh for uri {}", mesh_id);
                        continue;
                    }
                };

                let mesh_manager = scene.get_manager_mut::<MeshManager>(); // FIXME: No mutable borrows!
                mesh_manager.give_mesh(entity, gpu_mesh);
            }
        }
        // TODO: Apply the node's transform to the entity transform.

        // Instantiate each of the children and set the current node as their parent.
        for node in &node.children {
            let child = try!(self.instantiate_node(scene, node));

            let mut transform_manager = scene.get_manager_mut::<TransformManager>();
            transform_manager.set_child(entity, child);
        }

        Ok(entity)
    }

    pub fn get_shader<P: AsRef<Path> + ::std::fmt::Debug>(
        &self,
        shader_path: P
    ) -> Result<ShaderProgram, ParseShaderError> {
        {
            let path_str = shader_path.as_ref().to_str().expect(&*format!(
                "shader path {:?} contains invalid unicode characters",
                shader_path));
            if let Some(shader) = self.shaders.borrow().get(path_str) {
                return Ok(shader.clone());
            }
        }

        // This should be an else block on the above if block, but that doesn't work until MIR has
        // dropped, so for now we have to settle for manual case analysis.
        {
            let path_string: String = shader_path.as_ref()
                .to_str()
                .expect(&*format!(
                    "shader path {:?} contains invalid unicode characters",
                    shader_path))
                .into();

            let mut full_path = self.resource_path.borrow().clone();
            full_path.push(shader_path);
            let program_src = load_file_text(full_path);

            let programs = try!(ShaderParser::parse(&*program_src));
            let vert_src = match programs.iter().find(|program| program.name == "vert") {
                None => return Err(ParseShaderError::NoVertProgram),
                Some(program) => program.src,
            };

            let frag_src = match programs.iter().find(|program| program.name == "frag") {
                None => return Err(ParseShaderError::NoFragProgram),
                Some(program) => program.src,
            };

            let shader = self.renderer.compile_shader_program(vert_src, frag_src);
            self.shaders.borrow_mut().insert(path_string, shader.clone());

            Ok(shader)
        }
    }

    pub fn add_mesh(&self, uri: String, mesh: Mesh) {
        let mut meshes = self.meshes.borrow_mut();

        if meshes.contains_key(&uri) {
            println!("WARNING: There is already a mesh node with uri {}, it will be overriden in the resource manager by the new node", uri);
        }

        meshes.insert(uri.clone(), mesh);
    }

    pub fn add_mesh_node(&self, uri: String, node: MeshNode) {
        let mut nodes = self.mesh_nodes.borrow_mut();

        if nodes.contains_key(&uri) {
            println!("WARNING: There is already a mesh node with uri {}, it will be overriden in the resource manager by the new node", uri);
        }

        nodes.insert(uri.clone(), node);
    }

    fn has_cached_mesh(&self, uri: &str) -> bool {
        self.gpu_meshes.borrow().contains_key(uri)
    }

    fn get_cached_mesh(&self, uri: &str) -> Option<GLMeshData> {
        self.gpu_meshes
        .borrow()
        .get(uri)
        .map(|mesh| *mesh)
    }

    fn gen_gpu_mesh(&self, uri: &str) -> Option<GLMeshData> {
        // TODO: Don't do this check in release builds.
        if self.has_cached_mesh(uri) {
            println!("WARNING: Attempting to create a new mesh for {} when the uri is already in the meshes map", uri);
        }

        let meshes = self.meshes.borrow();
        let mesh = match meshes.get(uri) {
            Some(mesh) => mesh,
            None => return None,
        };

        let mesh_data = self.renderer.gen_mesh(mesh);
        self.gpu_meshes
        .borrow_mut()
        .insert(uri.into(), mesh_data);

        Some(mesh_data)
    }
}

pub fn load_file_text<P: AsRef<Path>>(file_path: P) -> String {
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.as_ref().display(), Error::description(&why)),
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", file_path.as_ref().display(), Error::description(&why)),
        Ok(_) => ()
    }
    contents
}

// TODO: Also include the node's local transform.
#[derive(Debug, Clone)]
pub struct MeshNode {
    pub mesh_ids: Vec<String>,
    pub children: Vec<MeshNode>,
}

impl MeshNode {
    pub fn new() -> MeshNode {
        MeshNode {
            mesh_ids: Vec::new(),
            children: Vec::new(),
        }
    }
}
