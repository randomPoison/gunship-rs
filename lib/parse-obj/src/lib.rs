use std::path::Path;
use std::slice;
use std::str::FromStr;

pub type Point = (f32, f32, f32, f32);
pub type Vector3 = (f32, f32, f32);

/// A parsed OBJ file.
///
/// TODO: Add a specialized implementation for `Debug` that does a better job pretty printing.
#[derive(Debug, Clone)]
pub struct Obj {
    positions: Vec<Point>,
    position_indices: Vec<Vec<usize>>,
    texcoords: Vec<Vector3>,
    texcoord_indices: Vec<Vec<usize>>,
    normals: Vec<Vector3>,
    normal_indices: Vec<Vec<usize>>,
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
        /// Pulls the next token and parses it as an `f32`.
        fn pull_f32(tokens: &mut Iterator<Item=&str>) -> Result<f32, Error> {
            let token = try!(tokens.next().ok_or(Error::MissingElement));
            let value = try!(f32::from_str(token));
            Ok(value)
        }

        /// Parses the next token as `f32` or returns `None`.
        ///
        /// Returns `Ok(None)` if no tokens are left in `tokens`, but will treat an empty token as
        /// an error.
        fn pull_option_f32(tokens: &mut Iterator<Item=&str>) -> Result<Option<f32>, Error> {
            match tokens.next() {
                Some(token) => {
                    let value = try!(f32::from_str(token));
                    Ok(Some(value))
                },
                None => {
                    Ok(None)
                }
            }
        }

        /// Parses the next token as 'usize', returning empty tokens as `None`.
        fn pull_option_usize(tokens: &mut Iterator<Item=&str>) -> Result<Option<usize>, Error> {
            let token = try!(tokens.next().ok_or(Error::MissingElement));
            if token == "" {
                Ok(None)
            } else {
                let value = try!(usize::from_str(token));
                Ok(Some(value))
            }
        }

        let mut positions = Vec::new();
        let mut position_indices = Vec::new();
        let mut texcoords = Vec::new();
        let mut texcoord_indices = Vec::new();
        let mut normals = Vec::new();
        let mut normal_indices = Vec::new();

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
                    let w = try!(pull_option_f32(&mut tokens)).unwrap_or(1.0);

                    positions.push((x, y, z, w));
                },

                // Vertex texcoord data.
                "vt" => {
                    let u = try!(pull_f32(&mut tokens));
                    let v = try!(pull_option_f32(&mut tokens)).unwrap_or(0.0);
                    let w = try!(pull_option_f32(&mut tokens)).unwrap_or(0.0);

                    texcoords.push((u, v, w));
                },

                // Vertex normal data.
                "vn" => {
                    let x = try!(pull_f32(&mut tokens));
                    let y = try!(pull_f32(&mut tokens));
                    let z = try!(pull_f32(&mut tokens));

                    normals.push((x, y, z));
                },

                // Indices for the face.
                "f" => {
                    let mut face_positions = Vec::new();
                    let mut face_texcoords = Vec::new();
                    let mut face_normals = Vec::new();

                    for vertex_str in tokens {
                        let mut index_tokens = vertex_str.split('/');

                        // Position index.
                        if let Some(index) = try!(pull_option_usize(&mut index_tokens)) {
                            face_positions.push(index - 1);
                        }

                        // Texcoord index.
                        if let Some(index) = try!(pull_option_usize(&mut index_tokens)) {
                            face_texcoords.push(index - 1);
                        }

                        // Normal index.
                        if let Some(index) = try!(pull_option_usize(&mut index_tokens)) {
                            face_normals.push(index - 1);
                        }
                    }

                    if face_texcoords.len() != 0 {
                        // The face has texcoord indices. There must be exactly one for each
                        // vertex or it's an error.
                        if face_texcoords.len() != face_positions.len() {
                            return Err(Error::MismatchedIndexData);
                        }

                        // Add face texcoords to the texcoords list.
                        texcoord_indices.push(face_texcoords);
                    }

                    if face_normals.len() != 0 {
                        // The face has normal indices. There must be exactly one for each vertex
                        // or it's an error.
                        if face_normals.len() != face_positions.len() {
                            return Err(Error::MismatchedIndexData);
                        }

                        // Add face normals to the normals list.
                        normal_indices.push(face_normals);
                    }

                    // All vertices must have position data.
                    if face_positions.len() == 0 {
                        return Err(Error::MissingPositionIndex);
                    }

                    position_indices.push(face_positions);
                },

                // TODO: Handle the case where there is no space between the '#' and the rest of
                // the comment (e.g. "#blah blah").
                "#" => {},

                // TODO: Implement these other directives.
                // TODO: Warn about unimplemented directives.
                "g" => {},
                "s" => {},
                "vp" => {},
                "p" => {},
                "l" => {},
                "o" => {},
                "mg" => {},
                "cstype" => {},
                "deg" => {},
                "bmat" => {},
                "step" => {},
                "curv" => {},
                "curv2" => {},
                "surv" => {},
                "parm" => {},
                "trim" => {},
                "hole" => {},
                "scrv" => {},
                "sp" => {},
                "end" => {},
                "con" => {},
                "bevel" => {},
                "c_interp" => {},
                "d_interp" => {},
                "lod" => {},
                "usemtl" => {},
                "shadow_obj" => {},
                "trace_obj" => {},
                "ctech" => {},
                "stech" => {},

                _ => {
                    return Err(Error::UnrecognizedDirective(line_beginning.into()));
                },
            }
        }

        // Check that either all of the faces of texcoords or none do.
        if texcoord_indices.len() != 0 && texcoord_indices.len() != position_indices.len() {
            return Err(Error::MismatchedFaceData);
        }

        // Check that either all of the faces of normals or none do.
        if normal_indices.len() != 0 && normal_indices.len() != position_indices.len() {
            return Err(Error::MismatchedFaceData);
        }

        Ok(Obj {
            positions: positions,
            position_indices: position_indices,
            texcoords: texcoords,
            texcoord_indices: texcoord_indices,
            normals: normals,
            normal_indices: normal_indices,
        })
    }

    /// Gets the list of vertex position tuples.
    pub fn positions(&self) -> &[Point] {
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

    /// Gets the list of position indices.
    ///
    /// Each face is represented as a list of indices
    pub fn position_indices(&self) -> &[Vec<usize>] {
        &*self.position_indices
    }

    /// Gets the list of texcoord tuples.
    pub fn texcoords(&self) -> &[Vector3] {
        &*self.texcoords
    }

    /// Gets the raw list of floating point values in the texcoords list.
    ///
    /// Usefule for sending texcoord data to the gpu.
    pub fn raw_texcoords(&self) -> &[f32] {
        use std::slice;

        let len = self.texcoords.len() * 3;
        let ptr = self.texcoords.as_ptr() as *const _;

        unsafe {
            slice::from_raw_parts(ptr, len)
        }
    }

    /// Gets the list of texcoord indices.
    pub fn texcoord_indices(&self) -> &[Vec<usize>] {
        &*self.texcoord_indices
    }

    /// Gets the list of normal tuples.
    pub fn normals(&self) -> &[Vector3] {
        &*self.normals
    }

    /// Gets the raw list of floating point values in the vertex normals list.
    ///
    /// Useful for sending vertex normal data to the gpu.
    pub fn raw_normals(&self) -> &[f32] {
        use std::slice;

        let len = self.normals.len() * 3;
        let ptr = self.normals.as_ptr() as *const _;

        unsafe {
            slice::from_raw_parts(ptr, len)
        }
    }

    /// Gets the list of normal indices.
    pub fn normal_indices(&self) -> &[Vec<usize>] {
        &*self.normal_indices
    }

    /// Returns an iterator over the faces in mesh.
    pub fn faces(&self) -> FaceIter {
        FaceIter {
            obj: self,

            position_faces: self.position_indices.iter(),
            texcoord_faces: self.texcoord_indices.iter(),
            normal_faces: self.normal_indices.iter(),
        }
    }
}

// TODO: Include line number and column in errors.
#[derive(Debug)]
pub enum Error {
    UnrecognizedDirective(String),

    /// Indicates that some faces have normal or texcoord data but others don't.
    MismatchedFaceData,

    /// Indicates that a face has normal or texcoord indices for some but not all of its vertices.
    ///
    /// According to the OBJ specification a face must have postion data for all of its vertices
    /// and may optionally have texcoord and normal data. If any of the vertices for a face has
    /// texcoord or normal data, though, all of the vertices must have that data. Therefore it is
    /// an error for one or more vertices in a face to have texcoord or normal data and one or
    /// vertices to not have that data.
    MismatchedIndexData,

    MissingDirectiveData,
    MissingElement,
    MissingPositionData,
    MissingPositionIndex,
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

/// An iterator over the vertices in a
pub struct Face<'a> {
    obj: &'a Obj,

    position_indices: slice::Iter<'a, usize>,
    texcoord_indices: Option<slice::Iter<'a, usize>>,
    normal_indices: Option<slice::Iter<'a, usize>>,
}

impl<'a> Iterator for Face<'a> {
    type Item = (Point, Option<Vector3>, Option<Vector3>);

    fn next(&mut self) -> Option<Self::Item> {
        self
            .position_indices
            .next()
            .map(|pos_index| {
                let pos = self.obj.positions[*pos_index];

                let tex =
                    self
                    .texcoord_indices
                    .as_mut()
                    .and_then(|mut texcoord_indices| texcoord_indices.next())
                    .map(|&index| self.obj.texcoords[index]);
                let norm =
                    self
                    .normal_indices
                    .as_mut()
                    .and_then(|mut normal_indices| normal_indices.next())
                    .map(|&index| self.obj.normals[index]);
                (pos, tex, norm)
            })
    }
}

/// An iterator over the faces in an OBJ file.
pub struct FaceIter<'a> {
    obj: &'a Obj,

    position_faces: slice::Iter<'a, Vec<usize>>,
    texcoord_faces: slice::Iter<'a, Vec<usize>>,
    normal_faces: slice::Iter<'a, Vec<usize>>,
}

impl<'a> Iterator for FaceIter<'a> {
    type Item = Face<'a>;

    fn next(&mut self) -> Option<Face<'a>> {
        self
            .position_faces
            .next()
            .map(|pos_face| {
                let tex_face = self.texcoord_faces.next().map(|indices| indices.iter());
                let norm_face = self.normal_faces.next().map(|indices| indices.iter());

                Face {
                    obj: self.obj,

                    position_indices: pos_face.iter(),
                    texcoord_indices: tex_face,
                    normal_indices: norm_face,
                }
            })
    }
}
