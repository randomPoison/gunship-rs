use std::ops::Index;

/// Represents a token in a material source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /* Keywords */
    Program,
    Property,

    /* Operator symbols */
    Eq,

    /* Structural symbols */
    Colon,
    SemiColon,
    OpenCurly,
    CloseCurly,

    /* Literal */
    ProgramLiteral,

    /* Name components */
    Identifier,

    /* Other */
    EndOfFile,
}

/// Represents a span covering a chunk of source material.
///
/// Used to reconstruct line numbers for errors. The indices are the byte indices in the source
/// document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub begin: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(begin: usize, end: usize) -> Span {
        Span {
            begin: begin,
            end: end,
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &str {
        &self[index.begin..index.end]
    }
}
