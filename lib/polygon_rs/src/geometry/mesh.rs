/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
#[derive(Debug)]
pub struct Mesh {
    pub raw_data: Vec<f32>,
    pub indices: Vec<u32>,
    pub position_attribute: VertexAttribute,
    pub normal_attribute: Option<VertexAttribute>,
    // pub uv_attribute:     Option<VertexAttribute>,
    // pub color_attribute:  Option<VertexAttribute>,
}

impl Mesh {
    /// Create a new mesh from existing data passed as slices.
    pub fn from_raw_data(positions_raw: &[f32], indices_raw: &[u32]) -> Mesh {
        let mut raw_data: Vec<f32> = Vec::with_capacity(positions_raw.len());
        raw_data.extend(positions_raw);

        let mut indices: Vec<u32> = Vec::with_capacity(indices_raw.len());
        indices.extend(indices_raw);

        Mesh {
            raw_data: raw_data,
            indices: indices,
            position_attribute: VertexAttribute {
                stride: 4,
                offset: 0,
            },
            normal_attribute: None,
        }
    }

    /// Adds the normals data to the mesh's raw data and creates the associated `VertexAttribute`.
    pub fn add_normals(&mut self, normals_raw: &[f32]) {
        self.normal_attribute = Some(VertexAttribute {
            stride: 3,
            offset: self.raw_data.len(),
        });
        self.raw_data.extend(normals_raw);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub stride: usize,
    pub offset: usize,
}
