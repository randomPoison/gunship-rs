use lexer::{Lexer, Error as TokenError};
use material_source::{MaterialSource, ProgramSource, PropertySource, PropertyType};
use token::*;

#[derive(Debug)]
pub struct Parser<'a> {
    source: &'a str,
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &str) -> Parser {
        Parser {
            source: source,
            lexer: Lexer::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<MaterialSource, Error> {
        let mut properties = Vec::new();
        let mut programs = Vec::new();

        loop {
            let (token, span) = self.lexer.next()?;
            match token {
                Token::Program => programs.push(self.parse_program(span)?),
                Token::Property => properties.push(self.parse_property(span)?),

                Token::EndOfFile => break,

                _ => return Err(Error::ExpectedItem(span)),
            }
        }

        Ok(MaterialSource {
            properties: properties,
            programs: programs,
        })
    }

    /// Parses a property item.
    ///
    /// # Preconditions
    ///
    /// - The "property" keyword was already pulled from the lexer.
    fn parse_property(&mut self, _start_span: Span) -> Result<PropertySource, Error> {
        let (token, span) = self.lexer.next()?;
        let ident = match token {
            Token::Identifier => self.source[span].into(),
            _ => return Err(Error::ExpectedIdent(span)),
        };

        let (token, span) = self.lexer.next()?;
        match token {
            Token::Colon => {},
            _ => return Err(Error::ExpectedColon(span)),
        }

        let (token, span) = self.lexer.next()?;
        let property_type = match token {
            Token::Identifier => match &self.source[span] {
                "Color" => PropertyType::Color,
                "Texture2d" => PropertyType::Texture2d,
                "f32" => PropertyType::f32,
                "Vector3" => PropertyType::Vector3,
                _ => return Err(Error::BadPropertyType(span)),
            },
            _ => return Err(Error::ExpectedIdent(span)),
        };

        let (token, span) = self.lexer.next()?;
        match token {
            Token::SemiColon => {},
            _ => return Err(Error::ExpectedSemiColon(span)),
        }

        Ok(PropertySource {
            name: ident,
            property_type: property_type,
        })
    }

    /// Parses a program item.
    ///
    /// # Preconditions
    ///
    /// - The "program" keyword was already pulled from the lexer.
    fn parse_program(&mut self, _start_span: Span) -> Result<ProgramSource, Error> {
        let (first_token, first_span) = self.lexer.next()?;
        if first_token != Token::Identifier {
            return Err(Error::ExpectedIdent(first_span));
        }

        let (second_token, second_span) = self.lexer.next()?;
        if second_token != Token::ProgramLiteral {
            return Err(Error::ExpectedProgramLiteral(second_span))
        }

        let program_literal = &self.source[second_span];
        let program_source = match &self.source[first_span] {
            "vert" => ProgramSource::Vertex(program_literal.into()),
            "frag" => ProgramSource::Fragment(program_literal.into()),
            _ => return Err(Error::BadProgramType(second_span)),
        };

        Ok(program_source)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    TokenError(TokenError),
    ExpectedItem(Span),
    ExpectedIdent(Span),
    ExpectedColon(Span),
    ExpectedProgramLiteral(Span),
    ExpectedSemiColon(Span),
    BadPropertyType(Span),
    BadProgramType(Span),
}

impl From<TokenError> for Error {
    fn from(from: TokenError) -> Error {
        Error::TokenError(from)
    }
}
