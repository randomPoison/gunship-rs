extern crate parse_collada as collada;

pub use self::collada::{ArrayElement, Collada, GeometricElement, Geometry, Node, PrimitiveElements, VisualScene};

use polygon::geometry::mesh::Mesh;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Error {
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

    UnsupportedGeometricElement,
    UnsupportedPrimitiveType,
    UnsupportedSourceElement,
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// Load the mesh data from a COLLADA .dae file.
///
/// The data in a COLLADA files is formatted for efficiency, but isn't necessarily
/// organized in a way that is supported by the graphics API. This method reformats the
/// data so that it can be sent straight to the GPU without further manipulation.
///
/// In order to to this, it reorganizes the normals, UVs, and other vertex attributes to
/// be in the same order as the vertex positions.
pub fn geometry_to_mesh(geometry: &Geometry) -> Result<Mesh> {
    match geometry.geometric_element {
        GeometricElement::Mesh(ref mesh) => collada_mesh_to_mesh(mesh),
        _ => Err(Error::UnsupportedGeometricElement),
    }
}

fn collada_mesh_to_mesh(mesh: &collada::Mesh) -> Result<Mesh> {
    let position_data_raw = try!(get_raw_positions(&mesh));

    let triangles = match mesh.primitive_elements[0] {
        PrimitiveElements::Triangles(ref triangles) => triangles,
        _ => return Err(Error::UnsupportedPrimitiveType),
    };

    let normal_data_raw = try!(get_normals_for_triangles(mesh, triangles)).unwrap(); // TODO: Handle the case where there's no normal data.
    let primitive_indices = &triangles.p.as_ref().unwrap();

    // Create a new array for the positions so we can add the w coordinate.
    let mut position_data: Vec<f32> = Vec::with_capacity(position_data_raw.len() / 3 * 4);

    // Create a new array for the normals and rearrange them to match the order of position attributes.
    let mut normal_data: Vec<f32> = Vec::with_capacity(position_data.len());

    // Iterate over the indices, rearranging the normal data to match the position data.
    let stride = mesh.source.len(); // TODO: Do we have a better way of calculating stride? What if one of the sources isn't used? OR USED TWICE!?
    let mut vertex_index_map: HashMap<(usize, usize), u32> = HashMap::new();
    let mut indices: Vec<u32> = Vec::new();
    let vertex_count = triangles.count * 3;
    let mut index_count = 0;
    for offset in 0..vertex_count {
        // Determine the offset of the the current vertex's attributes
        let position_index = primitive_indices[offset * stride];
        let normal_index = primitive_indices[offset * stride + 1];

        // Push the index of the vertex, either reusing an existing vertex or creating a new one.
        let vertex_key = (position_index, normal_index);
        let vertex_index = if vertex_index_map.contains_key(&vertex_key) {
            // Vertex has already been assembled, reuse the index.
            (*vertex_index_map.get(&vertex_key).unwrap()) as u32
        } else {
            // Assemble new vertex.
            let vertex_index = index_count;
            index_count += 1;
            vertex_index_map.insert(vertex_key, vertex_index as u32);

            // Append position to position data array.
            for offset in 0..3 {
                position_data.push(position_data_raw[position_index * 3 + offset]);
            }
            position_data.push(1.0);

            // Append normal to normal data array.
            for offset in 0..3 {
                normal_data.push(normal_data_raw[normal_index * 3 + offset]);
            }

            vertex_index
        };

        indices.push(vertex_index);
    }

    let mut mesh = Mesh::from_raw_data(position_data.as_ref(), indices.as_ref());
    mesh.add_normals(normal_data.as_ref());

    Ok(mesh)
}

/// Parses the <mesh> element to find the <source> element associated with the "POSITION" input semantic.
///
/// A <mesh> element is required to have a <source> element with the "POSITION" input semantic so
/// if the COLLADA document is well-formed this should never fail. Unfortunately parse-collada is
/// incomplete so it's still possible for a malformed document to parse successfully, hence why
/// this function returns a Result. In the future this may no longer be necessary.
fn get_raw_positions(mesh: &collada::Mesh) -> Result<&[f32]> {
    // The <vertices> element will always have the "POSITION" input.
    let pos_input = try!(
        mesh.vertices.input
        .iter()
        .find(|input| input.semantic == "POSITION")
        .ok_or(Error::MissingPositionSemantic));
    let pos_source_id = &*pos_input.source;

    // Find the <source> element with the "POSITION" data.
    let position_source = try!(
        mesh.source
        .iter()
        .find(|source| source.id == *pos_source_id)
        .ok_or(Error::MissingPositionSemantic));

    // Retrieve it's array_element, which is technically optional according to the spec but is
    // probably going to be there for the position data.
    let position_element = try!(
        position_source.array_element
        .as_ref()
        .ok_or(Error::MissingPositionData));

    // TODO: Do we care if position data is in any other format? My suspicion is that it will
    // only ever be a float array, but if that's not the case then we need to support other
    // formats even if they're uncommon.
    let position_data: &[f32] = match *position_element {
        ArrayElement::Float(ref float_array) => float_array.contents.as_ref(),
        _ => return Err(Error::UnsupportedSourceElement),
    };

    Ok(position_data)
}

fn get_normals_for_triangles<'a>(mesh: &'a collada::Mesh, triangles: &'a collada::Triangles) -> Result<Option<&'a [f32]>> {
    // First we have to find the input with the correct semantic.
    // Check the mesh's <vertices> element first.
    let source_id = {
        // Retrieve the id specified by the <input> element with the "NORMAL" semantic.
        let source_id =
            mesh.vertices.input
            .iter()
            .find(|input| input.semantic == "NORMAL")
            .map(|input| &input.source)
            .or_else(||
                triangles.input
                .iter()
                .find(|input| input.semantic == "NORMAL")
                .map(|input| &input.source)
            );

        // If no input has the "NORMAL" semnatic then the mesh/triangles has no normal, so we
        // can return None.
        match source_id {
            Some(id) => id,
            None => return Ok(None),
        }
    };

    // Find source for normal data.
    let source = try!(
        mesh.source
        .iter()
        .find(|source| source.id == **source_id)
        .ok_or(Error::MissingSourceData));

    // Retrieve it's array_element, which is technically optional according to the spec but is
    // probably going to be there for the normal data.
    let element: &collada::ArrayElement = try!(
        source.array_element
        .as_ref()
        .ok_or(Error::MissingNormalData));

    // TODO: Do we care if normal data is in any other format? My suspicion is that it will
    // only ever be a float array, but if that's not the case then we need to support other
    // formats even if they're uncommon.
    let normal_data = match *element {
        ArrayElement::Float(ref float_array) => float_array.contents.as_ref(),
        _ => return Err(Error::UnsupportedSourceElement),
    };

    Ok(Some(normal_data))
}
