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
    texcoord_data: Vec<Vec<Vector2>>,

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

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.position_data.push(vertex.position);

        if let Some(normal) = vertex.normal {
            self.normal_data.push(normal);
        }

        // Ensure there is a list for each texcoord.
        while self.texcoord_data.len() < vertex.texcoord.len() {
            self.texcoord_data.push(Vec::new());
        }

        // Add each texcoord to its corresponding list.
        let iter = vertex.texcoord.iter().zip(self.texcoord_data.iter_mut());
        for (texcoord, texcoord_data) in iter {
            texcoord_data.push(*texcoord);
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

    pub fn set_texcoord_data(mut self, texcoord_data: &[&[Vector2]]) -> MeshBuilder {
        if texcoord_data.len() == 0 {
            return self;
        }

        for (src, dest) in texcoord_data.iter().zip(self.texcoord_data.iter_mut()) {
            dest.clear();
            dest.extend(*src);
        }

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
            return Err(BuildMeshError::IncorrectAttributeCount);
        }

        for texcoord_set in &self.texcoord_data {
            if texcoord_set.len() != vertex_count {
                return Err(BuildMeshError::IncorrectAttributeCount);
            }
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

        let float_count = {
            let mut float_count =
                self.position_data.len() * 4
              + self.normal_data.len() * 3;
            for texcoord_set in &self.texcoord_data {
                float_count += texcoord_set.len() * 2;
            }

            float_count
        };

        // Create the mesh.
        let mut vertex_data = Vec::<f32>::with_capacity(float_count);

        // Setup position data.
        let position_attrib = VertexAttribute {
            offset: 0,
            stride: 4,
        };
        vertex_data.extend(Point::as_ref(&*self.position_data));

        // Setup normal data.
        let normal_attrib = if self.normal_data.len() > 0 {
            let attrib = VertexAttribute {
                offset: vertex_data.len(),
                stride: 3,
            };
            vertex_data.extend(Vector3::as_ref(&*self.normal_data));

            Some(attrib)
        } else {
            None
        };

        // Setup texcoord data.
        let mut texcoord_attribs = Vec::new();
        for texcoord_set in &self.texcoord_data {
            texcoord_attribs.push(VertexAttribute {
                offset: vertex_data.len(),
                stride: 2,
            });
            vertex_data.extend(Vector2::as_ref(&*texcoord_set));
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
