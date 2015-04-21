use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use collada::{ColladaData, GeometricElement, ArrayElement, PrimitiveType};

use polygon::gl_render::{GLRender, GLMeshData};
use polygon::geometry::mesh::Mesh;
use polygon::geometry::face::Face;

use math::point::Point;

use entity::Entity;

pub struct MeshManager {
    meshes: Vec<GLMeshData>
}

impl MeshManager {
    pub fn new() -> MeshManager {
        MeshManager {
            meshes: Vec::new()
        }
    }

    pub fn create(&mut self, entity: Entity, renderer: &GLRender, path_text: &str) -> &GLMeshData {
        self.meshes.push(load_mesh(renderer, path_text));
        let index = self.meshes.len() - 1;
        &self.meshes[index]
    }

    pub fn meshes(&self) -> &Vec<GLMeshData> {
        &self.meshes
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

pub fn load_mesh(renderer: &GLRender, path_text: &str) -> GLMeshData {
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

    let mesh = match collada_data.library_geometries.geometries[0].data {
        GeometricElement::Mesh(ref mesh) => mesh,
        _ => panic!("What even is this shit?")
    };

    let vertex_data_raw: &[f32] = match mesh.sources[0].array_element {
        ArrayElement::Float(ref float_array)  => {
            float_array.as_ref()
        },
        _ => panic!("Thas some bullshit.")
    };
    assert!(vertex_data_raw.len() > 0);

    let mut vertex_data: Vec<Point> = Vec::new();
    for offset in (0..vertex_data_raw.len() / 3) {
        vertex_data.push(Point::from_slice(&vertex_data_raw[offset * 3..offset * 3 + 3]));
    }
    assert!(vertex_data.len() > 0);

    let triangles = match mesh.primitives[0] {
        PrimitiveType::Triangles(ref triangles) => triangles,
        _ => panic!("This isn't even cool.")
    };

    let stride = triangles.inputs.len();
    let face_data_raw = triangles.primitives.iter().enumerate().filter_map(|(index, &value)| {
            if index % stride == 0 {
                Some(value as u32)
            } else {
                None
            }
        }).collect::<Vec<u32>>();
    assert!(face_data_raw.len() > 0);

    let mut face_data: Vec<Face> = Vec::new();
    for offset in (0..face_data_raw.len() / 3) {
        face_data.push(Face::from_slice(&face_data_raw[offset * 3..offset * 3 + 3]));
    }
    assert!(face_data.len() > 0);

    let mesh = Mesh::from_slice(vertex_data.as_ref(), face_data.as_ref());

    let frag_src = load_file("shaders/test3D.frag.glsl");
    let vert_src = load_file("shaders/test3D.vert.glsl");

    renderer.gen_mesh(&mesh,
                      vert_src.as_ref(),
                      frag_src.as_ref())
}
