use math::{Point, Vector3};
use geometry::face::Face;

/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
pub struct Mesh {
    pub raw_data: Vec<f32>,
    pub indices: Vec<u32>,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: VertexAttribute,
}

impl Mesh {
    /// Create a new mesh from existing data passed as slices.
    pub fn from_raw_data(positions_raw: &[f32], normals_raw: &[f32], indices_raw: &[u32]) -> Mesh {
        let mut raw_data: Vec<f32> = Vec::with_capacity(positions_raw.len() + normals_raw.len());
        raw_data.push_all(positions_raw);
        raw_data.push_all(normals_raw);

        let mut indices: Vec<u32> = Vec::with_capacity(indices_raw.len());
        indices.push_all(indices_raw);

        Mesh {
            raw_data: raw_data,
            indices: indices,
            position_attribute: VertexAttribute {
                stride: 4,
                offset: 0,
            },
            normal_attribute: VertexAttribute {
                stride: 3,
                offset: positions_raw.len() as u32,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub stride: u32,
    pub offset: u32,
}
