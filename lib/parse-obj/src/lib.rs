use std::path::Path;
use std::str::FromStr;

// TODO: Add a specialized implementation for `Debug` that does a better job pretty printing.
#[derive(Debug, Clone)]
pub struct Obj {
    positions: Vec<(f32, f32, f32, f32)>,
    faces: Vec<Vec<usize>>,
}

impl Obj {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Obj, Error> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = try!(File::open(path));
        let mut text = String::new();

        try!(file.read_to_string(&mut text));

        Obj::from_str(&text)
    }

    pub fn from_str(file_text: &str) -> Result<Obj, Error> {
        fn pull_f32(token: &mut Iterator<Item=&str>) -> Result<f32, Error> {
            let text = try!(token.next().ok_or(Error::MissingElement));
            let value = try!(f32::from_str(text));
            Ok(value)
        }

        let mut positions = Vec::new();
        let mut faces = Vec::new();

        for line in file_text.lines() {
            let mut tokens = line.split_whitespace();
            let line_beginning = match tokens.next() {
                Some(token) => token,
                None => continue, // Line is empty, skip it.
            };

            match line_beginning {
                // Vertex position data.
                "v" => {
                    let x = try!(pull_f32(&mut tokens));
                    let y = try!(pull_f32(&mut tokens));
                    let z = try!(pull_f32(&mut tokens));
                    let w = 1.0; // TOOO: Actually pull from the input.

                    positions.push((x, y, z, w));
                },

                // TODO: Implement these other directives.
                "vt" => {},
                "vn" => {},
                "g" => {},
                "s" => {},

                // Indices for the face.
                "f" => {
                    let mut vertices = Vec::new();

                    for vertex_str in tokens {
                        let index_str = try!(
                            vertex_str
                            .split('/')
                            .next()
                            .ok_or(Error::MissingDirectiveData));
                        let index = try!(usize::from_str(index_str));
                        vertices.push(index);
                    }

                    faces.push(vertices);
                },

                // TODO: Handle the case where there is no space between the '#' and the rest of
                // the comment (e.g. "#blah blah").
                "#" => {},

                _ => {
                    return Err(Error::UnrecognizedDirective(line_beginning.into()));
                },
            }
        }

        Ok(Obj {
            positions: positions,
            faces: faces,
        })
    }

    /// Gets the list of vertex position tuples.
    pub fn positions(&self) -> &[(f32, f32, f32, f32)] {
        &*self.positions
    }

    /// Gets the raw list of floating point values in the vertex positions list.
    ///
    /// Useful for sending vertex position data to the gpu.
    pub fn raw_positions(&self) -> &[f32] {
        use std::slice;

        let len = self.positions.len() * 4;
        let ptr = self.positions.as_ptr() as *const _;

        unsafe {
            slice::from_raw_parts(ptr, len)
        }
    }

    /// Gets the list of face indices.
    ///
    /// Each face is represented as a list of indices
    pub fn faces(&self) -> &[Vec<usize>] {
        &*self.faces
    }
}

// TODO: Include line number and column in errors.
#[derive(Debug)]
pub enum Error {
    UnrecognizedDirective(String),
    MissingDirectiveData,
    MissingElement,
    ParseFloatError(::std::num::ParseFloatError),
    ParseIntError(::std::num::ParseIntError),
    IoError(::std::io::Error),
}

impl From<::std::num::ParseFloatError> for Error {
    fn from(error: ::std::num::ParseFloatError) -> Error {
        Error::ParseFloatError(error)
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(error: ::std::num::ParseIntError) -> Error {
        Error::ParseIntError(error)
    }
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Error {
        Error::IoError(error)
    }
}
