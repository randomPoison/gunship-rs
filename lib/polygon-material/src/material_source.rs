use lexer::{Lexer, Error as LexError};
use std::fs::File;
use std::io::Error as IoError;
use std::io::prelude::*;
use std::path::Path;

/// Represents the contents of a material file that has been loaded into memory but has not been
/// sent to the renderer.
#[derive(Debug)]
pub struct MaterialSource {
    properties: Vec<PropertySource>,
    programs: Vec<ProgramSource>,
}

impl MaterialSource {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<MaterialSource, Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        MaterialSource::from_str(&*contents)
    }

    pub fn from_str(source: &str) -> Result<MaterialSource, Error> {
        let mut lexer = Lexer::new(source);

        unimplemented!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProgramSource {
    Vertex(String),
    Fragment(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(bad_style)]
pub enum PropertySource {
    Color,
    Texture,
    f32,
}

/// Represents an error in parsing a material source file.
pub enum Error {
    IoError(IoError),
    LexError(LexError),
}

impl From<LexError> for Error {
    fn from(error: LexError) -> Error {
        Error::LexError(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IoError(error)
    }
}
