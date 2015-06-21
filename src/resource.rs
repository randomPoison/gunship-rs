use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::rc::Rc;

use collada::{self, ColladaData, GeometricElement, ArrayElement, PrimitiveType};

use polygon::gl_render::{GLRender, GLMeshData};
use polygon::geometry::mesh::Mesh;

use wav::Wave;

#[derive(Debug, Clone)]
pub struct ResourceManager {
    renderer: GLRender,
    meshes: HashMap<String, GLMeshData>,
    audio_clips: HashMap<String, Rc<Wave>>,
}

impl ResourceManager {
    pub fn new(renderer: GLRender) -> ResourceManager {
        ResourceManager {
            renderer: renderer,
            meshes: HashMap::new(),
            audio_clips: HashMap::new(),
        }
    }

    pub fn get_mesh(&mut self, path_text: &str) -> GLMeshData {
        if self.meshes.contains_key(path_text) {
            *self.meshes.get(path_text).unwrap()
        }
        else
        {
            let frag_src = load_file("shaders/forward_phong.frag.glsl");
            let vert_src = load_file("shaders/forward_phong.vert.glsl");

            let mesh = COLLADALoader::load_from_file(path_text);
            let mesh_data =
                self.renderer.gen_mesh(&mesh, vert_src.as_ref(), frag_src.as_ref());

            self.meshes.insert(path_text.to_string(), mesh_data);
            mesh_data
        }
    }

    pub fn get_audio_clip(&mut self, path_text: &str) -> Rc<Wave> {
        if !self.audio_clips.contains_key(path_text) {
            let wave = Wave::from_file(path_text).unwrap();
            self.audio_clips.insert(path_text.into(), Rc::new(wave));
        }

        self.audio_clips.get(path_text).unwrap().clone()
    }
}

struct COLLADALoader;

impl COLLADALoader {
    fn load_from_file(path_text: &str) -> Mesh {
        // load data from COLLADA file
        let file_path = Path::new(path_text);
        let mut file = match File::open(&file_path) {
            // The `desc` field of `IoError` is a string that describes the error
            Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
            Ok(file) => file,
        };
        let collada_data = match ColladaData::from_file(&mut file) {
            Err(why) => panic!(why),
            Ok(data) => data
        };

        COLLADALoader::parse(collada_data)
    }

    /// Load the mesh data from a COLLADA .dae file.
    ///
    /// The data in a COLLADA files is formatted for efficiency, but isn't necessarily
    /// organized in a way that is supported by the graphics API. This method reformats the
    /// data so that it can be sent straight to the GPU without further manipulation.
    ///
    /// In order to to this, it reorganizes the normals, UVs, and other vertex attributes to
    /// be in the same order as the vertex positions.
    fn parse(collada_data: ColladaData) -> Mesh {
        let mesh = match collada_data.library_geometries.geometries[0].data {
            GeometricElement::Mesh(ref mesh) => mesh,
            _ => panic!("No mesh found within geometry")
        };

        let position_data_raw = COLLADALoader::get_raw_positions(&mesh);
        let normal_data_raw = COLLADALoader::get_normals(&mesh);

        let triangles = match mesh.primitives[0] {
            PrimitiveType::Triangles(ref triangles) => triangles,
            _ => panic!("Only triangles primitives are supported currently")
        };
        let primitive_indices = &triangles.primitives;

        // Create a new array for the positions so we can add the w coordinate.
        let mut position_data: Vec<f32> = Vec::with_capacity(position_data_raw.len() / 3 * 4);

        // Create a new array for the normals and rearrange them to match the order of position attributes.
        let mut normal_data: Vec<f32> = Vec::with_capacity(position_data.len());

        // Iterate over the indices, rearranging the normal data to match the position data.
        let stride = triangles.inputs.len();
        let mut vertex_index_map: HashMap<(usize, usize), u32> = HashMap::new();
        let mut indices: Vec<u32> = Vec::new();
        let vertex_count = triangles.count * 3;
        let mut index_count = 0;
        for offset in 0..vertex_count {
            // Determine the offset of the the current vertex's attributes
            let position_index = primitive_indices[offset * stride];
            let normal_index = primitive_indices[offset * stride + 1];

            // Push the index of the vertex, either reusing an existing vertex or creating a new one.
            let vertex_key = (position_index, normal_index);
            let vertex_index = if vertex_index_map.contains_key(&vertex_key) {
                // Vertex has already been assembled, reuse the index.
                (*vertex_index_map.get(&vertex_key).unwrap()) as u32
            } else {
                // Assemble new vertex.
                let vertex_index = index_count;
                index_count += 1;
                vertex_index_map.insert(vertex_key, vertex_index as u32);

                // Append position to position data array.
                for offset in 0..3 {
                    position_data.push(position_data_raw[position_index * 3 + offset]);
                }
                position_data.push(1.0);

                // Append normal to normal data array.
                for offset in 0..3 {
                    normal_data.push(normal_data_raw[normal_index * 3 + offset]);
                }

                vertex_index
            };

            indices.push(vertex_index);
        }

        let mesh = Mesh::from_raw_data(position_data.as_ref(), normal_data.as_ref(), indices.as_ref());

        mesh
    }

    fn get_raw_positions(mesh: &collada::Mesh) -> &[f32] {
        // TODO: Consult the correct element (<triangles> for now) to determine which source has position data.
        let position_data: &[f32] = match mesh.sources[0].array_element {
            ArrayElement::Float(ref float_array) => float_array.as_ref(),
            _ => panic!("Only float arrays supported for vertex position array")
        };
        assert!(position_data.len() > 0);

        position_data
    }

    fn get_normals(mesh: &collada::Mesh) -> &[f32] {
        // TODO: Consult the correct element (<triangles> for now) to determine which source has normal data.
        let normal_data: &[f32] = match mesh.sources[1].array_element {
            ArrayElement::Float(ref float_array) => float_array.as_ref(),
            _ => panic!("Only float arrays supported for vertex normal array")
        };
        assert!(normal_data.len() > 0);

        normal_data
    }
}

pub fn load_file(path: &str) -> String {
    let file_path = Path::new(path);
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", file_path.display(), Error::description(&why)),
        Ok(_) => ()
    }
    contents
}
