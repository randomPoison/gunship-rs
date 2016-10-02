use async::engine::{self, RenderMessage};
use polygon::geometry::mesh::{BuildMeshError, MeshBuilder};
use polygon::math::Vector2;
use obj::{self, Obj};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod collada;

static MESH_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static MATERIAL_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Load all data from the specified file as an array of bytes.
pub fn load_file_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path)?;

    let mut bytes = if let Ok(metadata) = file.metadata() {
        Vec::with_capacity(metadata.len() as usize)
    } else {
        Vec::new()
    };

    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Load all data from the specified file as a `String`.
pub fn load_file_text<P: AsRef<Path>>(path: P) -> Result<String, LoadTextError> {
    let bytes = load_file_bytes(path)?;
    String::from_utf8(bytes).map_err(|utf8_err| utf8_err.into())
}

#[derive(Debug)]
pub enum LoadTextError {
    Io(io::Error),
    Utf8(FromUtf8Error),
}

impl From<io::Error> for LoadTextError {
    fn from(from: io::Error) -> LoadTextError {
        LoadTextError::Io(from)
    }
}

impl From<FromUtf8Error> for LoadTextError {
    fn from(from: FromUtf8Error) -> LoadTextError {
        LoadTextError::Utf8(from)
    }
}

/// Loads a mesh from disk.
///
/// Loads a mesh data from the specified path and performs any necessary processing to prepare it
/// to be used in rendering.
pub fn load_mesh<P: AsRef<Path>>(path: P) -> Result<Mesh, LoadMeshError> {
    let path = path.as_ref();

    // Load mesh source and parse mesh data based on file type.
    let mesh_data = match path.extension() {
        Some(ext) if ext == "dae" => {
            let text = load_file_text(path)?;
            collada::load_resources(text)?
        },
        Some(ext) if ext == "obj" => {
            let text = load_file_text(path)?;

            // Load mesh file and normalize indices for OpenGL.
            let obj = Obj::from_str(&*text)?;

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
                .build()?
        },
        _ => {
            let path = path
                .as_os_str()
                .to_string_lossy()
                .into_owned();
            return Err(LoadMeshError::UnsupportedFileType(path));
        },
    };

    // Create handle for mesh data and asynchronously register it with the renderer.
    let mesh_id = MESH_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    engine::send_render_message(RenderMessage::Mesh(mesh_id, mesh_data));

    Ok(Mesh(mesh_id))
}

pub type MeshId = usize;

#[derive(Debug)]
pub struct Mesh(MeshId);

impl Mesh {
    // TODO: Make this private to the crate.
    pub fn id(&self) -> MeshId {
        self.0
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        // TODO: How do we cleanup a mesh?
    }
}

#[derive(Debug)]
pub enum LoadMeshError {
    BuildMeshError(BuildMeshError),
    LoadTextError(LoadTextError),
    ParseColladaError(collada::Error),
    ParseObjError(obj::Error),
    UnsupportedFileType(String),
}

impl From<BuildMeshError> for LoadMeshError {
    fn from(from: BuildMeshError) -> LoadMeshError {
        LoadMeshError::BuildMeshError(from)
    }
}

impl From<LoadTextError> for LoadMeshError {
    fn from(from: LoadTextError) -> LoadMeshError {
        LoadMeshError::LoadTextError(from)
    }
}

impl From<collada::Error> for LoadMeshError {
    fn from(from: collada::Error) -> LoadMeshError {
        LoadMeshError::ParseColladaError(from)
    }
}

impl From<obj::Error> for LoadMeshError {
    fn from(from: obj::Error) -> LoadMeshError {
        LoadMeshError::ParseObjError(from)
    }
}

pub fn load_material<P: AsRef<Path>>(path: P) -> Result<Material, LoadMaterialError> {
    // Load and parse material data.
    let text = load_file_text(path)?;
    let material_source = ::polygon::material::MaterialSource::from_str(text)?;

    let material_id = MATERIAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    engine::send_render_message(RenderMessage::Material(material_id, material_source));

    Ok(Material(material_id))
}

pub type MaterialId = usize;

#[derive(Debug)]
pub struct Material(MaterialId);

#[derive(Debug)]
pub enum LoadMaterialError {
    LoadTextError(LoadTextError),
    BuildMaterialError(::polygon::BuildMaterialError),
    ParseMaterialError(::polygon::material::MaterialSourceError),
}

impl From<LoadTextError> for LoadMaterialError {
    fn from(from: LoadTextError) -> LoadMaterialError {
        LoadMaterialError::LoadTextError(from)
    }
}

impl From<::polygon::material::MaterialSourceError> for LoadMaterialError {
    fn from(from: ::polygon::material::MaterialSourceError) -> LoadMaterialError {
        LoadMaterialError::ParseMaterialError(from)
    }
}

impl From<::polygon::BuildMaterialError> for LoadMaterialError {
    fn from(from: ::polygon::BuildMaterialError) -> LoadMaterialError {
        LoadMaterialError::BuildMaterialError(from)
    }
}
