use parser::{Parser, Error as ParseError};
use std::fs::File;
use std::io::Error as IoError;
use std::io::prelude::*;
use std::path::Path;

/// Represents the contents of a material file that has been loaded into memory but has not been
/// sent to the renderer.
#[derive(Debug, PartialEq, Eq)]
pub struct MaterialSource {
    pub properties: Vec<PropertySource>,
    pub programs: Vec<ProgramSource>,
}

impl MaterialSource {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<MaterialSource, Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        MaterialSource::from_str(&*contents)
    }

    pub fn from_str<T: AsRef<str>>(source: T) -> Result<MaterialSource, Error> {
        let mut parser = Parser::new(source.as_ref());
        parser.parse().map_err(|error| error.into())
    }
}

/// Represents a program item parsed from a material file.
///
/// TODO: Document the different variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProgramSource {
    Vertex(String),
    Fragment(String),
}

impl ProgramSource {
    /// Checks if the program source is a vertex shader.
    pub fn is_vertex(&self) -> bool {
        match *self {
            ProgramSource::Vertex(_) => true,
            _ => false,
        }
    }

    /// Checks if the programs source is fragment shader.
    pub fn is_fragment(&self) -> bool {
        match *self {
            ProgramSource::Fragment(_) => true,
            _ => false,
        }
    }

    pub fn source(&self) -> &str {
        match *self {
            ProgramSource::Vertex(ref source) => &*source,
            ProgramSource::Fragment(ref source) => &*source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertySource {
    pub name: String,
    pub property_type: PropertyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(bad_style)]
pub enum PropertyType {
    Color,
    Texture2d,
    f32,
    Vector3,
}

/// Represents an error in parsing a material source file.
#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    ParseError(ParseError),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match *self {
            Error::IoError(_) => false,
            Error::ParseError(parse_error) => match *other {
                Error::IoError(_) => false,
                Error::ParseError(other_parse_error) => parse_error == other_parse_error
            }
        }
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::ParseError(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IoError(error)
    }
}
