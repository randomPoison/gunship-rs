extern crate parse_obj;

use polygon::geometry::mesh::*;
use self::parse_obj::*;
use std::path::Path;

pub fn load_mesh<P: AsRef<Path>>(path: P) -> Result<Mesh, BuildMeshError> {
    // Load mesh file and normalize indices for OpenGL.
    let obj = Obj::from_file(path).unwrap();

    // Gather vertex data so that OpenGL can use them.
    let mut positions = Vec::new();
    let mut normals = Vec::new();

    // Iterate over each of the faces in the mesh.
    let face_indices =
        obj
        .position_indices()
        .iter()
        .zip(obj.normal_indices().iter());
    for (face_position_indices, face_normal_indices) in face_indices {
        // Iterate over each of the vertices in the face to combine the position and normal into
        // a single vertex.
        for (position_index, normal_index) in
            face_position_indices.iter().zip(face_normal_indices.iter()) {
            let position = obj.positions()[*position_index];
            positions.push(position.into());

            let normal = obj.normals()[*normal_index];
            normals.push(normal.into());
        }
    }

    // Create indices list.
    let indices_count = obj.position_indices().len() as u32 * 3;
    let indices: Vec<u32> = (0..indices_count).collect();

    MeshBuilder::new()
        .set_position_data(&*positions)
        .set_normal_data(&*normals)
        .set_indices(&*indices)
        .build()
}
