/// Implements the logic behind converting COLLADA documents to polygon-rs meshes.

extern crate parse_collada as collada;

use math::*;
use polygon::geometry::mesh::*;
pub use self::collada::{
    AnyUri,
    ArrayElement,
    Collada,
    GeometricElement,
    Geometry,
    Node,
    PrimitiveElements,
    UriFragment,
    VisualScene
};

#[derive(Debug)]
pub enum Error {
    /// Indicates an error that occurred when the MeshBuilder was validating the mesh data. If the
    /// COLLADA document passed parsing this should not occur.
    BuildMeshError(BuildMeshError),

    IncorrectPrimitiveIndicesCount {
        primitive_count: usize,
        stride: usize,
        index_count: usize,
    },

    /// Indicates an error in loading or parsing the original collada document (i.e. the error
    /// ocurred within the parse-collada library).
    ParseColladaError(collada::Error),

    /// Indicates that there was an input with the "NORMAL" semantic but the associated source
    /// was missing.
    MissingNormalSource,

    /// Indicates that an <input> element specified a <source> element that was missing.
    MissingSourceData,

    /// Indicates that the <source> element with the "POSITION" semantic was missing an
    /// array element.
    MissingPositionData,

    /// Indicates that the <source> element with the "NORMAL" semantic was missing an array element.
    MissingNormalData,

    /// Indicates that a <vertices> element had and <input> element with no "POSITION" semantic.
    ///
    /// NOTE: This error means that the COLLADA document is ill-formed and should have failed
    /// parsing. This indicates that there is a bug in the parse-collada library that should be
    /// fixed.
    MissingPositionSemantic,

    /// Indicates that the <mesh> had no primitive elements.
    MissingPrimitiveElement,

    /// Indicates that one of the primitive elements (e.g. <trianges> et al) were missing a <p>
    /// child element. While this is technically allowed by the standard I'm not really sure what
    /// to do with that? Like how do you define a mesh without indices?
    MissingPrimitiveIndices,

    /// Indicates that a uri referenced an asset outside the document.
    NonLocalUri(String),

    UnsupportedGeometricElement,
    UnsupportedPrimitiveType,

    /// Indicates that a <source> element's array element was of a type other than <float_array>.
    UnsupportedSourceData,
}

impl From<collada::Error> for Error {
    fn from(from: collada::Error) -> Error {
        Error::ParseColladaError(from)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum VertexSemantic {
    Position,
    Normal,
    TexCoord,
}

/// Loads all resources from a COLLADA document and adds them to the resource manager.
pub fn load_resources<T: Into<String>>(source: T) -> Result<Mesh> {
    let collada_data = Collada::parse(source)?;

    // Load all meshes from the document and add them to the resource manager.
    if let Some(library_geometries) = collada_data.library_geometries {
        for geometry in library_geometries.geometry {
            // // Retrieve the id for the geometry.
            // // TODO: Generate an id for the geometry if it doesn't already have one.
            // let id = match geometry.id {
            //     None => {
            //         println!("WARNING: COLLADA file contained a <geometry> element with no \"id\" attribute");
            //         println!("WARNING: This is unsupported because there is no way to reference that geometry to instantiate it");
            //         continue;
            //     },
            //     Some(id) => id,
            // };

            let mesh = match geometry.geometric_element {
                GeometricElement::Mesh(ref mesh) => try!(collada_mesh_to_mesh(mesh)),
                _ => return Err(Error::UnsupportedGeometricElement),
            };

            // TODO: Actually finish parsing all the other data from the file.
            return Ok(mesh);
        }
    }

    unimplemented!();
}

fn collada_mesh_to_mesh(mesh: &collada::Mesh) -> Result<Mesh> {
    if mesh.primitive_elements.len() > 1 {
        println!("WARNING: Mesh is composed of more than one geometric primitive, which is not currently supported, only part of the mesh will be loaded");
    }

    // Grab the first primitive element in the mesh.
    // TODO: Handle all primitive elements in the mesh, not just one. This is dependent on polygon
    // being able to support submeshes.
    let primitive = try!(
        mesh.primitive_elements.first()
        .ok_or(Error::MissingPrimitiveElement));

    let triangles = match *primitive {
        PrimitiveElements::Triangles(ref triangles) => triangles,
        _ => return Err(Error::UnsupportedPrimitiveType),
    };

    let primitive_indices =
        triangles.p
        .as_ref()
        .ok_or(Error::MissingPrimitiveIndices)?;

    // Iterate over the indices, rearranging the normal data to match the position data.
    let stride = triangles.input.len(); // TODO: Do we have a better way of calculating stride? What if one of the sources isn't used? OR USED TWICE!?
    let count  = triangles.count;
    let index_count = primitive_indices.len();
    let vertex_count = count as u32 * 3;

    // Verify we have the right number of indices to build the vertices.
    if count * stride * 3 != index_count {
        return Err(Error::IncorrectPrimitiveIndicesCount {
            primitive_count: count,
            stride: stride,
            index_count: index_count,
        });
    }

    // The indices list is a just a raw list of indices. They are implicity grouped based on the
    // number of inputs for the primitive element (e.g. if there are 3 inputs for the primitive
    // then there are 3 indices per vertex). To handle this we use GroupBy to do a strided
    // iteration over indices list and build each vertex one at a time. Internally the mesh
    // builder handles the details of how to assemble the vertex data in memory.

    // Build a mapping between the vertex indices and the source that they use.
    let mut source_map = Vec::new();
    for (offset, input) in triangles.input.iter().enumerate() {
        // Retrieve the approriate source. If the semantic is "VERTEX" then the offset is
        // associated with all of the sources specified by the <vertex> element.
        let source_ids = match &*input.semantic {
            "VERTEX" => {
                mesh.vertices.input
                .iter()
                .map(|input| (input.semantic.as_ref(), input.source.as_ref()))
                .collect()
            },
            _ => vec![(input.semantic.as_ref(), input.source.as_ref())],
        };

        // For each of the semantics at the current offset, push their info into the source map.
        for (semantic, source_id) in source_ids {
            // Retrieve the <source> element for the input.
            let source = try!(mesh.source
            .iter()
            .find(|source| source.id == source_id)
            .ok_or(Error::MissingSourceData));

            // Retrieve it's array_element, which is technically optional according to the spec but is
            // probably going to be there for the position data.
            let array_element = try!(
                source.array_element
                .as_ref()
                .ok_or(Error::MissingPositionData));

            // Get float data. Raw mesh data should only be float data (the only one that even
            // remotely makes sense is int data, and even then that seems unlikely), so emit an
            // error if the data is in the wrong format.
            let data = match *array_element {
                ArrayElement::Float(ref float_array) => float_array.contents.as_ref(),
                _ => return Err(Error::UnsupportedSourceData),
            };

            source_map.push(IndexMapper {
                offset: offset,
                semantic: semantic,
                data: data,
            });
        }
    }

    let mut mesh_builder = MeshBuilder::new();
    let mut unsupported_semantic_flag = false;
    for vertex_indices in GroupBy::new(primitive_indices, stride).unwrap() { // TODO: This can't fail... right? I'm pretty sure the above checks make sure this is correct.
        // We iterate over each group of indices where each group represents the indices for a
        // single vertex. Within that vertex we need
        let mut vertex = Vertex::new(Point::origin());

        for (offset, index) in vertex_indices.iter().enumerate() {
            for mapper in source_map.iter().filter(|mapper| mapper.offset == offset) {
                match mapper.semantic {
                    "POSITION" => {
                        vertex.position = Point::new(
                            // TODO: Don't assume that the position data is encoded as 3 coordinate
                            // vectors. The <technique_common> element for the source should have
                            // an <accessor> describing how the data is laid out.
                            mapper.data[index * 3 + 0],
                            mapper.data[index * 3 + 1],
                            mapper.data[index * 3 + 2],
                        );
                    },
                    "NORMAL" => {
                        vertex.normal = Some(Vector3::new(
                            mapper.data[index * 3 + 0],
                            mapper.data[index * 3 + 1],
                            mapper.data[index * 3 + 2],
                        ));
                    },
                    "TEXCOORD" => {
                        vertex.texcoord.push(Vector2::new(
                            mapper.data[index * 2 + 0],
                            mapper.data[index * 2 + 1],
                        ));
                    },
                    _ => if !unsupported_semantic_flag {
                        unsupported_semantic_flag = true;
                        println!("WARNING: Unsupported vertex semantic {} in mesh will not be used", mapper.semantic);
                    },
                }
            }
        }

        mesh_builder.add_vertex(vertex);
    }

    let indices: Vec<u32> = (0..vertex_count).collect();

    mesh_builder
    .set_indices(&*indices)
    .build()
    .map_err(|err| Error::BuildMeshError(err))
}

struct IndexMapper<'a> {
    offset:   usize,
    semantic: &'a str,
    data:     &'a [f32],
}

// TODO: Where even should this live? It's generally useful but I'm only using it here right now.
struct GroupBy<'a, T: 'a> {
    next:     *const T,
    end:      *const T,
    stride:   usize,
    _phantom: ::std::marker::PhantomData<&'a T>,
}

impl<'a, T: 'a> GroupBy<'a, T> {
    fn new(slice: &'a [T], stride: usize) -> ::std::result::Result<GroupBy<'a, T>, ()> {
        if slice.len() % stride != 0 {
            return Err(());
        }

        Ok(GroupBy {
            next: slice.as_ptr(),
            end: unsafe { slice.as_ptr().offset(slice.len() as isize) },
            stride: stride,
            _phantom: ::std::marker::PhantomData,
        })
    }
}

impl<'a, T: 'a> Iterator for GroupBy<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {
        if self.next == self.end {
            return None;
        }

        let next = self.next;
        self.next = unsafe { self.next.offset(self.stride as isize) };

        Some(unsafe {
            ::std::slice::from_raw_parts(next, self.stride)
        })
    }
}
