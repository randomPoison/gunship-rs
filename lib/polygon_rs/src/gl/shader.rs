use std::fs::File;
use std::io::Error as IoError;
use std::io::prelude::*;
use std::path::Path;
use super::gl_util::{Program, ProgramError, Shader, ShaderError, ShaderType};

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Program, Error> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    from_str(&*contents)
}

pub fn from_str(source: &str) -> Result<Program, Error> {
    let programs = parse_programs(source)?;

    let vert_src =
        programs
        .iter()
        .find(|program| program.name == "vert")
        .ok_or(Error::NoVertProgram)
        ?;

    let frag_src =
        programs
        .iter()
        .find(|program| program.name == "frag")
        .ok_or(Error::NoFragProgram)
        ?;

    let vert_shader = Shader::new(vert_src.src, ShaderType::Vertex)?;
    let frag_shader = Shader::new(frag_src.src, ShaderType::Fragment)?;
    let program = Program::new(&[vert_shader, frag_shader])?;

    Ok(program)
}

#[derive(Debug)]
pub enum Error {
    CompileError(ShaderError),
    IoError(IoError),
    LinkError(ProgramError),
    MissingOpeningBrace,
    MultipleVertShader,
    MultipleFragShader,
    NoFragProgram,
    NoVertProgram,
    ProgramMissingName,
    UnmatchedBraces,
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IoError(error)
    }
}

impl From<ShaderError> for Error {
    fn from(error: ShaderError) -> Error {
        Error::CompileError(error)
    }
}

impl From<ProgramError> for Error {
    fn from(error: ProgramError) -> Error {
        Error::LinkError(error)
    }
}

#[derive(Debug, Clone)]
struct ShaderProgramSrc<'a> {
    pub name: &'a str,
    pub src: &'a str,
}

fn parse_programs(shader_src: &str) -> Result<Vec<ShaderProgramSrc>, Error> {
    let mut programs: Vec<ShaderProgramSrc> = Vec::new();
    let mut index = 0;
    loop {
        let substr = &shader_src[index..];
        let (program, end_index) = parse_program(substr)?;
        programs.push(program);
        index = end_index;

        if programs.len() >= 2 {
            break;
        }
    }

    Ok(programs)
}

fn parse_program(src: &str) -> Result<(ShaderProgramSrc, usize), Error> {
    if let Some(index) = src.find("program") {
        let program_src = src[index..].trim_left();
        let program_name =
            program_src
            .split_whitespace()
            .nth(1)
            .ok_or(Error::ProgramMissingName)?;

        let (program_src, end_index) = match program_src.find('{') {
            None => return Err(Error::MissingOpeningBrace),
            Some(index) => {
                let (src, index) = try!(parse_braces_contents(&program_src[index..]));
                (src.trim(), index)
            }
        };

        let program = ShaderProgramSrc {
            name: program_name,
            src: program_src,
        };
        Ok((program, end_index))
    } else {
        return Err(Error::NoVertProgram);
    }
}

/// Parses the contents of a curly brace-delimeted block.
///
/// Retuns a substring of the source string that contains the contents of the block without
/// the surrounding curly braces. Fails if there is no matching close brace.
fn parse_braces_contents(src: &str) -> Result<(&str, usize), Error> {
    assert!(src.starts_with("{"));

    let mut depth = 0;
    for (index, character) in src.chars().enumerate() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    // We're at the end.
                    return Ok((&src[1..index], index));
                }
            },
            _ => {}
        }
    }

    // Uh-oh, we got to the end and never closed the braces.
    Err(Error::UnmatchedBraces)
}
