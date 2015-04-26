use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use collada::{self, ColladaData, GeometricElement, ArrayElement, PrimitiveType};

use math::{Point, Vector3};

use polygon::gl_render::{GLRender, GLMeshData};
use polygon::geometry::mesh::Mesh;
use polygon::geometry::face::Face;

pub struct ResourceManager {
    renderer: GLRender,
    meshes: HashMap<String, GLMeshData>,
}

impl ResourceManager {
    pub fn new(renderer: GLRender) -> ResourceManager {
        ResourceManager {
            renderer: renderer,
            meshes: HashMap::new(),
        }
    }

    pub fn get(&mut self, path_text: &str) -> GLMeshData {
        if self.meshes.contains_key(path_text) {
            *self.meshes.get(path_text).unwrap()
        }
        else
        {
            let frag_src = load_file("shaders/test3D.frag.glsl");
            let vert_src = load_file("shaders/test3D.vert.glsl");

            let mesh = COLLADALoader::load_from_file(path_text);
            let mesh_data =
                self.renderer.gen_mesh(&mesh, vert_src.as_ref(), frag_src.as_ref());

            self.meshes.insert(path_text.to_string(), mesh_data);
            mesh_data
        }
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
        position_data.resize(position_data_raw.len() / 3 * 4, 0.0);

        for offset in 0..position_data_raw.len() / 3 {
            // Copy the position from the source array and add the w coordinate.
            let source_offset = offset * 3;
            let dest_offset = offset * 4;
            position_data[dest_offset + 0] = position_data_raw[source_offset + 0];
            position_data[dest_offset + 1] = position_data_raw[source_offset + 1];
            position_data[dest_offset + 2] = position_data_raw[source_offset + 2];
            position_data[dest_offset + 3] = 1.0;
        }

        // Create a new array for the normals and rearrange them to match the order of position attributes.
        let mut sorted_normals: Vec<f32> = Vec::with_capacity(position_data.len());
        sorted_normals.resize(position_data.len(), 0.0);

        // Iterate over the indices, rearranging the normal data to match the position data.
        let stride = triangles.inputs.len();
        let mut indices: Vec<u32> = Vec::new();
        for offset in 0..(triangles.count * 3) {
            // Determine the offset of the the current vertex's attributes
            let source_offset = primitive_indices[offset * stride];
            let normal_offset = primitive_indices[offset * stride + 1];

            // Copy the normal from the source array to the correct location in the sorted array.
            // sorted_normals[source_offset + 0] = normal_data_raw[normal_offset + 0];
            // sorted_normals[source_offset + 1] = normal_data_raw[normal_offset + 1];
            // sorted_normals[source_offset + 2] = normal_data_raw[normal_offset + 2];

            indices.push(primitive_indices[offset * stride] as u32);
        }

        println!("position data: {:?}", position_data);
        println!("normal data: {:?}", sorted_normals);
        println!("indices: {:?}", indices);

        let mesh = Mesh::from_raw_data(position_data.as_ref(), sorted_normals.as_ref(), indices.as_ref());

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
