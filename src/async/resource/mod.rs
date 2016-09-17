use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::string::FromUtf8Error;

pub mod collada;

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

    let text = load_file_text(path)?;
    let mesh = collada::load_resources(text)?;

    // let mesh_id = Engine::renderer(|renderer| {
    //     renderer.register_mesh(&mesh)
    // });

    println!("registered mesh with id: {:?}", mesh);

    println!("Done with load mesh");
    Ok(mesh)
}

// #[derive(Debug)]
pub type Mesh = ::polygon::geometry::mesh::Mesh;

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

    let text = load_file_text(path)?;
    let material_source = ::polygon::material::MaterialSource::from_str(text)?;
    // let result = Engine::renderer(move |renderer| {
    //     let material = renderer.build_material(material_source)?;
    //     let material_id = renderer.register_material(material);
    //     Ok(material_id)
    // });

    println!("end load material");
    Ok(material_source)
}

// #[derive(Debug)]
pub type Material = ::polygon::material::MaterialSource;

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
