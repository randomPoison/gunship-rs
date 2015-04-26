use math::{Point, Vector3};
use geometry::face::Face;

/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl Mesh {
    /// Create a new Mesh with no data in it.
    #[allow(dead_code)]
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Create a new mesh from existing data passed as slices.
    pub fn from_slice(vert_data: &[Point], face_data: &[Face], normal_data: &[Vector3]) -> Mesh {
        let mut vertices: Vec<Vertex> = Vec::new();
        for (point, normal) in vert_data.iter().zip(normal_data.iter()) {
            let vertex = Vertex {
                position: *point,
                normal: *normal,
            };

            vertices.push(vertex);
        }


        let mut normals: Vec<Vector3> = Vec::new();
        for normal in normal_data {
            normals.push(*normal);
        }

        let mut faces: Vec<Face> = Vec::new();
        for face in face_data {
            faces.push(Face {
                indices:
                    [face.indices[0],
                     face.indices[1],
                     face.indices[2]]
            });
        }

        Mesh {
            vertices: vertices,
            faces: faces,
        }
    }
}

#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: Point,
    normal: Vector3,
}
