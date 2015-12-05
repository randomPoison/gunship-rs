use math::*;

pub type MeshIndex = u32;

/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
#[derive(Debug)]
pub struct Mesh {
    vertex_data: Vec<f32>,
    indices:     Vec<MeshIndex>,

    position: VertexAttribute,
    normal:   Option<VertexAttribute>,
    texcoord: Vec<VertexAttribute>,
}

impl Mesh {
    pub fn vertex_data(&self) -> &[f32] {
        &*self.vertex_data
    }

    pub fn indices(&self) -> &[MeshIndex] {
        &*self.indices
    }

    pub fn position(&self) -> VertexAttribute {
        self.position
    }

    pub fn normal(&self) -> Option<VertexAttribute> {
        self.normal
    }

    pub fn texcoord(&self) -> &[VertexAttribute] {
        &*self.texcoord
    }
}

/// Represents a single vertex in a mesh with all of its supported attributes.
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Point,
    pub normal: Option<Vector3>,

    /// Support an arbitrary number of texture units. The actual maximum is dependent on hardware
    /// and so is not limited by polygon directly. If the number of
    pub texcoord: Vec<Vector2>,
}

impl Vertex {
    pub fn new(position: Point) -> Vertex {
        Vertex {
            position: position,
            normal: None,
            texcoord: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub offset: usize,
    pub stride: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum BuildMeshError {
    IndexOutOfBounds {
        vertex_count: MeshIndex,
        index: MeshIndex,
    },

    /// Indicates that one or more attributes had a count that did not match the total number of
    /// vertices.
    IncorrectAttributeCount,
}

#[derive(Debug, Clone)]
pub struct MeshBuilder {
    position_data: Vec<Point>,
    normal_data: Vec<Vector3>,
    texcoord_data: Vec<Vec<Vector3>>,

    indices:  Vec<u32>,
}

// TODO: I'd like to support building the mesh by specifying all of each attribute at once, since
// that seems like a common use case for game development. We still want to support building it
// vert-by-vert because that works best in other situations (e.g. COLLADA files).
impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            position_data: Vec::new(),
            normal_data:   Vec::new(),
            texcoord_data: Vec::new(),
            indices:       Vec::new(),
        }
    }

    pub fn add_vertex(mut self, vertex: Vertex) -> MeshBuilder {
        self.position_data.push(vertex.position);

        if let Some(normal) = vertex.normal {
            self.normal_data.push(normal);
        }

        // TODO: Handle texcoord data.

        self
    }

    pub fn add_index(mut self, index: MeshIndex) -> MeshBuilder {
        self.indices.push(index);
        self
    }

    pub fn set_position_data(mut self, position_data: &[Point]) -> MeshBuilder {
        self.position_data.clear();
        self.position_data.extend(position_data);
        self
    }

    pub fn set_normal_data(mut self, normal_data: &[Vector3]) -> MeshBuilder {
        self.normal_data.clear();
        self.normal_data.extend(normal_data);
        self
    }

    pub fn set_texcoord_data(mut self, _texcoord_data: &[Vector2], _texcoord_index: MeshIndex) -> MeshBuilder {
        // TODO

        self
    }

    pub fn set_indices(mut self, indices: &[u32]) -> MeshBuilder {
        self.indices.clear();
        self.indices.extend(indices);
        self
    }

    pub fn build(self) -> Result<Mesh, BuildMeshError> {
        // Validate the mesh data.

        // The vertex count is defined by the position data, since position is the only required
        // vertex attribute.
        let vertex_count = self.position_data.len();

        // TODO: Validate texcoord data.
        if self.normal_data.len() != 0 && self.normal_data.len() != vertex_count {
            return Err(BuildMeshError::IncorrectAttributeCount);
        }

        // Make sure all indices at least point to a valid vertex.
        for index in self.indices.iter().cloned() {
            if index >= vertex_count as MeshIndex {
                return Err(BuildMeshError::IndexOutOfBounds {
                    vertex_count: vertex_count as MeshIndex,
                    index: index,
                });
            }
        }

        // TODO: Make sure all normals are normalized?

        // Create the mesh.
        let mut vertex_data =
            Vec::<f32>::with_capacity(
                self.position_data.len() * 4
              + self.normal_data.len() * 3);
        vertex_data.extend(Point::as_ref(&*self.position_data));
        vertex_data.extend(Vector3::as_ref(&*self.normal_data));

        // TODO: Add texcoord data.

        Ok(Mesh {
            vertex_data: vertex_data,
            indices: self.indices,

            position: VertexAttribute {
                offset: 0,
                stride: 4,
            },

            normal: if self.normal_data.len() > 0 {
                Some(VertexAttribute {
                    offset: self.position_data.len() * 4,
                    stride: 3,
                })
            } else {
                None
            },

            texcoord: Vec::new(),
        })
    }
}
