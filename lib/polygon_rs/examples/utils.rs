extern crate parse_bmp;
extern crate parse_obj;

use polygon::geometry::mesh::*;
use polygon::math::Vector2;
use polygon::texture::Texture2d;
use self::parse_bmp::Bitmap;
use self::parse_obj::*;
use std::path::Path;

pub fn load_mesh<P: AsRef<Path>>(path: P) -> Result<Mesh, BuildMeshError> {
    // Load mesh file and normalize indices for OpenGL.
    let obj = Obj::from_file(path).unwrap();

    // Gather vertex data so that OpenGL can use them.
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut texcoords = Vec::new();

    // Iterate over each of the faces in the mesh.
    for face in obj.faces() {
        // Iterate over each of the vertices in the face to combine the position and normal into
        // a single vertex.
        for (position, maybe_tex, maybe_normal) in face {
            positions.push(position.into());

            // NOTE: The w texcoord is provided according to the bitmap spec but we don't need to
            // use it here, so we simply ignore it.
            if let Some((u, v, _w)) = maybe_tex {
                texcoords.push(Vector2::new(u, v));
            }

            if let Some(normal) = maybe_normal {
                normals.push(normal.into());
            }
        }
    }

    // Create indices list.
    let indices_count = obj.position_indices().len() as u32 * 3;
    let indices: Vec<u32> = (0..indices_count).collect();

    MeshBuilder::new()
        .set_position_data(&*positions)
        .set_normal_data(&*normals)
        .set_texcoord_data(&*texcoords)
        .set_indices(&*indices)
        .build()
}

pub fn load_texture<P: AsRef<Path>>(path: P) -> Texture2d {
    let bitmap = Bitmap::load(path).unwrap();
    Texture2d::from_bitmap(bitmap)
}
