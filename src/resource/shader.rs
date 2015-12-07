#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseShaderError {
    NoVertProgram,
    NoFragProgram,
    MultipleVertShader,
    MultipleFragShader,
    ProgramMissingName,
    UnmatchedBraces,
    MissingOpeningBrace,
    CompileError(String),
    LinkError(String),
}

#[derive(Debug, Clone)]
pub struct ShaderParser;

#[derive(Debug, Clone)]
pub struct ShaderProgramSrc<'a> {
    pub name: &'a str,
    pub src: &'a str,
}

impl ShaderParser {
    pub fn parse(shader_src: &str) -> Result<Vec<ShaderProgramSrc>, ParseShaderError> {
        let mut programs: Vec<ShaderProgramSrc> = Vec::new();
        let mut index = 0;
        loop {
            let substr = &shader_src[index..];
            let (program, end_index) = try!(ShaderParser::parse_program(substr));
            programs.push(program);
            index = end_index;

            if programs.len() >= 2 {
                break;
            }
        }

        Ok(programs)
    }

    fn parse_program(src: &str) -> Result<(ShaderProgramSrc, usize), ParseShaderError> {
        if let Some(index) = src.find("program") {
            let program_src = src[index..].trim_left();
            let program_name = match program_src.split_whitespace().nth(1) {
                Some(name) => name,
                None => return Err(ParseShaderError::ProgramMissingName),
            };

            let (program_src, end_index) = match program_src.find('{') {
                None => return Err(ParseShaderError::MissingOpeningBrace),
                Some(index) => {
                    let (src, index) = try!(ShaderParser::parse_braces_contents(&program_src[index..]));
                    (src.trim(), index)
                }
            };

            let program = ShaderProgramSrc {
                name: program_name,
                src: program_src,
            };
            Ok((program, end_index))
        } else {
            return Err(ParseShaderError::NoVertProgram);
        }
    }

    /// Parses the contents of a curly brace-delimeted block.
    ///
    /// Retuns a substring of the source string that contains the contents of the block without
    /// the surrounding curly braces. Fails if there is no matching close brace.
    fn parse_braces_contents(src: &str) -> Result<(&str, usize), ParseShaderError> {
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
        Err(ParseShaderError::UnmatchedBraces)
    }
}
