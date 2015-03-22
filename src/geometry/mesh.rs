use math::point::Point;
use geometry::face::Face;

/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub faces: Vec<Face>
}

impl Mesh {
    /// Create a new Mesh with no data in it.
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            faces: Vec::new()
        }
    }

    /// Create a new mesh from existing data passed as slices.
    pub fn from_slice(vert_data: &[Point], face_data: &[Face]) -> Mesh {
        let mut vert_data_vec: Vec<Point> = Vec::new();
        for point in vert_data {
            vert_data_vec.push(Point {
                x: point.x,
                y: point.y,
                z: point.z,
                w: point.w
            });
        }

        let mut face_data_vec: Vec<Face> = Vec::new();
        for face in face_data {
            face_data_vec.push(Face {
                indices:
                    [face.indices[0],
                     face.indices[1],
                     face.indices[2]]
            });
        }

        Mesh {
            vertices: vert_data_vec,
            faces: face_data_vec
        }
    }
}
