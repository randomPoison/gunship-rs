use async::engine::{self, RenderMessage};
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
    // Load mesh source and parse mesh data.
    let text = load_file_text(path)?;
    let mesh_data = collada::load_resources(text)?;

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
    LoadTextError(LoadTextError),
    ParseColladaError(collada::Error),
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
