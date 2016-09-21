use polygon::GpuMesh;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::string::FromUtf8Error;
use std::sync::atomic::*;
use std::sync::mpsc::Sender;

pub mod collada;

thread_local! {
    // TODO: We don't want this to be completely public, only pub(crate), but `thread_local`
    // doesn't support pub(crate) syntax.
    pub static RENDER_MESSAGE_CHANNEL: RefCell<Option<Sender<RenderResourceMessage>>> = RefCell::new(None);
}

static MESH_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static MATERIAL_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub enum RenderResourceMessage {
    Mesh(MeshId, ::polygon::geometry::mesh::Mesh),
    Material(MaterialId, ::polygon::material::MaterialSource),
}

fn send_render_message(message: RenderResourceMessage) {
    RENDER_MESSAGE_CHANNEL.with(move |channel| {
        let borrow = channel.borrow();
        let channel = borrow.as_ref().expect("Render message channel was `None`");
        channel
            .send(message)
            .expect("Unable to send render resource message");
    });
}

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
    println!("start load mesh");

    // Load mesh source and parse mesh data.
    let text = load_file_text(path)?;
    let mesh_data = collada::load_resources(text)?;

    // Create handle for mesh data and asynchronously register it with the renderer.
    let mesh_id = MESH_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    send_render_message(RenderResourceMessage::Mesh(mesh_id, mesh_data));

    println!("Done with load mesh");
    Ok(Mesh(mesh_id))
}

type MeshId = usize;

#[derive(Debug)]
pub struct Mesh(MeshId);

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
    println!("start load material");

    // Load and parse material data.
    let text = load_file_text(path)?;
    let material_source = ::polygon::material::MaterialSource::from_str(text)?;

    let material_id = MATERIAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    send_render_message(RenderResourceMessage::Material(material_id, material_source));

    println!("end load material");
    Ok(Material(material_id))
}

type MaterialId = usize;

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
