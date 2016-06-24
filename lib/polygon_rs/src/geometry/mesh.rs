use math::*;

pub type MeshIndex = u32;

/// The raw data representing a mesh in memory.
///
/// Meshes are represented as list of vertex positions and a list of faces.
/// Each face is represented as 3 indices into the vertex array.
#[derive(Debug, Clone)]
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

/// A struct describing the single attribute within a mesh's vertex buffer.
#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    /// The number of elements in the attribute.
    pub elements: usize,

    /// The offset in elements from the start of the buffer.
    pub offset: usize,

    /// The stride in elements between consecutive vertices.
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
    IncorrectAttributeCount {
        attribute: VertexAttributeType,
        expected: usize,
        actual: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum VertexAttributeType {
    Position,
    Normal,
    Texcoord,
}

/// Provides a safe interface for building a mesh from raw vertex data.
///
/// `MeshBuilder` supports two methods for specifying mesh data: Providing a series of `Vertex`
/// objects, or directly setting the data for each vertex attribute at once. Once all vertex data
/// has been set calling `build()` will validate the mesh data and produce a `Mesh` object.
///
/// The mesh builder performs validity tests when building the mesh and mesh compilation can fail
/// if the mesh data is invalid in some way. The following validity tests are performed:
///
/// - Check for different data count for different attributes (e.g. if the position attribute data
///   for a different number of elements than the normal attribute).
/// - Any of the indicies would be out of bounds for the given vertex data.
#[derive(Debug, Clone)]
pub struct MeshBuilder {
    position_data: Vec<Point>,
    normal_data: Vec<Vector3>,
    texcoord_data: Vec<Vector2>,

    indices:  Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            position_data: Vec::new(),
            normal_data:   Vec::new(),
            texcoord_data: Vec::new(),
            indices:       Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.position_data.push(vertex.position);

        if let Some(normal) = vertex.normal {
            self.normal_data.push(normal);
        }

        assert!(vertex.texcoord.len() <= 1, "More than one texcoord per vertex is currently not supported");

        // Add each texcoord to its corresponding list.
        if vertex.texcoord.len() > 0 {
            self.texcoord_data.push(vertex.texcoord[0])
        }
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

    pub fn set_texcoord_data(mut self, texcoord_data: &[Vector2]) -> MeshBuilder {
        self.texcoord_data.clear();
        self.texcoord_data.extend(texcoord_data);
        self
    }

    pub fn set_indices(mut self, indices: &[u32]) -> MeshBuilder {
        self.indices.clear();
        self.indices.extend(indices);
        self
    }

    pub fn build(self) -> Result<Mesh, BuildMeshError> {
        // The vertex count is defined by the position data, since position is the only required
        // vertex attribute.
        let vertex_count = self.position_data.len();

        // Check that there is enough attribute data for each one.
        if self.normal_data.len() != 0 && self.normal_data.len() != vertex_count {
            return Err(BuildMeshError::IncorrectAttributeCount {
                attribute: VertexAttributeType::Normal,
                expected: vertex_count,
                actual: self.normal_data.len(),
            });
        }

        if self.texcoord_data.len() != 0 && self.texcoord_data.len() != vertex_count {
            return Err(BuildMeshError::IncorrectAttributeCount {
                attribute: VertexAttributeType::Texcoord,
                expected: vertex_count,
                actual: self.texcoord_data.len(),
            });
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

        // TODO: Check for degenerate triangles? Actually, should that be a failure or a warning?

        let float_count =
            self.position_data.len() * 4
          + self.normal_data.len() * 3
          + self.texcoord_data.len() * 2;

        // Create the mesh.
        let mut vertex_data = Vec::<f32>::with_capacity(float_count);

        // Setup position data.
        let position_attrib = VertexAttribute {
            elements: 4,
            offset: 0,
            stride: 0,
        };
        vertex_data.extend(Point::as_ref(&*self.position_data));

        // Setup normal data.
        let normal_attrib = if self.normal_data.len() > 0 {
            let attrib = VertexAttribute {
                elements: 3,
                offset: vertex_data.len(),
                stride: 0,
            };
            vertex_data.extend(Vector3::as_ref(&*self.normal_data));

            Some(attrib)
        } else {
            None
        };

        // Setup texcoord data.
        let mut texcoord_attribs = Vec::new();
        if self.texcoord_data.len() > 0 {
            texcoord_attribs.push(VertexAttribute {
                elements: 2,
                offset: vertex_data.len(),
                stride: 0,
            });
            vertex_data.extend(Vector2::as_ref(&*self.texcoord_data));
        }

        // By our powers combined! We are! A mesh.
        Ok(Mesh {
            vertex_data: vertex_data,
            indices: self.indices,

            position: position_attrib,
            normal: normal_attrib,
            texcoord: texcoord_attribs,
        })
    }
}
