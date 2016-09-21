use std::iter::Peekable;
use std::str::*;
use super::token::*;

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    is_done: bool,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            source: source,
            chars: source.char_indices().peekable(),
            is_done: false,
        }
    }

    pub fn next(&mut self) -> Result<(Token, Span)> {
        // Start by eating all whitespace before the next valid token.
        let (start_index, character) = {
            let mut start = None;
            while let Some((index, character)) = self.chars.next() {
                if !character.is_whitespace() {
                    // This is the first character of the current token. Mark the starting index and
                    // break the loop so we can begin reading the token.
                    start = Some((index, character));
                    break;
                }
            }

            match start {
                Some(index_char) => index_char,
                None => {
                    self.is_done = true;

                    // Never found a non-whitespace character, so we're at the end of the file.
                    let span = Span::new(self.source.len(), self.source.len());
                    return Ok((Token::EndOfFile, span))
                },
            }
        };

        // See if token is an identifier.
        if character.is_ident() {
            return self.parse_ident(start_index)
        }

        // See if token is a number literal.
        if character.is_numeric_part() {
            return unimplemented!();
        }

        // See if character is string literal.
        if character == '"' {
            return unimplemented!();
        }

        // See if token is program literal.
        if character == '{' {
            return self.parse_program_literal(start_index);
        }

        // Single-character symbols.
        let token = match character {
            ';' => Token::SemiColon,
            '=' => Token::Eq,
            ':' => Token::Colon,

            _ => {
                self.is_done = true;

                return Err(Error {
                    span: Span::new(start_index, start_index + 1),
                    data: ErrorData::IllegalSymbol(character),
                });
            },
        };

        Ok((token, Span::new(start_index, start_index + 1)))
    }

    /// Checks if the lexer is done.
    ///
    /// Returns `true` if the lexer reached the end of the file or found a token error.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    fn parse_ident(&mut self, start_index: usize) -> Result<(Token, Span)> {
        while let Some(&(end_index, character)) = self.chars.peek() {
            if !character.is_ident() {
                let word = &self.source[start_index .. end_index];
                let span = Span::new(start_index, end_index);

                let token = match word {
                    "property" => Token::Property,
                    "program" => Token::Program,
                    _ => Token::Identifier,
                };

                return Ok((token, span))
            }

            // Consume the item we peeked at.
            self.chars.next();
        }

        Ok((Token::Identifier, Span::new(start_index, self.source.len())))
    }

    fn parse_program_literal(&mut self, start_index: usize) -> Result<(Token, Span)> {
        // Start at depth 1 because we've already removed the opening '{'.
        let mut depth = 1;

        // Walk through the source string
        while let Some((end_index, character)) = self.chars.next() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        // We're at the end.
                        return Ok((Token::ProgramLiteral, Span::new(start_index + 1, end_index)));
                    }
                },
                _ => {}
            }
        }

        // Uh-oh, we got to the end and never closed the braces.
        self.is_done = true;
        Err(Error {
            span: Span::new(start_index, self.source.len()),
            data: ErrorData::UnclosedProgramLiteral,
        })
    }
}

/// Represents a lex error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    pub span: Span,
    pub data: ErrorData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorData {
    IllegalSymbol(char),
    UnclosedProgramLiteral,
}

trait CharacterParseExt {
    fn is_ident_start(self) -> bool;
    fn is_ident(self) -> bool;
    fn is_numeric_part(self) -> bool;
}

impl CharacterParseExt for char {
    fn is_ident_start(self) -> bool {
        self.is_alphabetic() || self == '_'
    }

    fn is_ident(self) -> bool {
        self.is_alphanumeric() || self == '_'
    }

    fn is_numeric_part(self) -> bool {
        self.is_numeric() || self == '-'
    }
}
