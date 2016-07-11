#![feature(const_fn)]

extern crate polygon_material as material;

use material::lexer::{Error as TokenError, ErrorData, Lexer};
use material::material_source::{PropertySource, ProgramSource, MaterialSource, Error as MaterialSourceError};
use material::parser::Error as ParseError;
use material::token::*;

/// Helper function for verifying the output of the lexer.
///
/// Takes a source string an array of expected outputs and verifies that the two are the same.
fn verify_lexer(
    source: &str,
    tokens_expected: &[Result<(Token, &'static str), (ErrorData, &'static str)>],
    material_expected: Result<MaterialSource, MaterialSourceError>,
) {
    println!("Tokens:");
    let mut lexer = Lexer::new(source);
    for tokens_expected in tokens_expected {
        let actual = lexer
        .next()
        .map(|(token, span)| (token, &source[span.begin..span.end]))
        .map_err(|error| (error.data, &source[error.span.begin..error.span.end]));
        println!("\t{:?}", actual);
        assert_eq!(*tokens_expected, actual);
    }
    assert!(lexer.is_done());

    let material_actual = MaterialSource::from_str(source);
    assert_eq!(material_expected, material_actual);
}

#[test]
fn lex_properties() {
    static SOURCE: &'static str = r#"
        property surface_color : Color ;
        property another_thing: f32;
        property some_vec:Vector3;
    "#;

    static EXPECTED_TOKENS: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Property, "property")),
        Ok((Token::Identifier, "surface_color")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "Color")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::Property, "property")),
        Ok((Token::Identifier, "another_thing")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "f32")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::Property, "property")),
        Ok((Token::Identifier, "some_vec")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "Vector3")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::EndOfFile, "")),
    ];

    let EXPECTED_MATERIAL = Ok(MaterialSource {
        properties: vec![
            PropertySource::Color,
            PropertySource::f32,
            PropertySource::Vector3,
        ],
        programs: vec![],
    });

    verify_lexer(SOURCE, EXPECTED_TOKENS, EXPECTED_MATERIAL);
}

#[test]
fn lex_sybmol_error() {
    static SOURCE: &'static str = r#"
        property prog: &&
    "#;

    static EXPECTED_TOKENS: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Property, "property")),
        Ok((Token::Identifier, "prog")),
        Ok((Token::Colon, ":")),
        Err((ErrorData::IllegalSymbol('&'), "&")),
    ];

    let EXPECTED_MATERIAL = Err(MaterialSourceError::ParseError(ParseError::TokenError(TokenError {
        span: Span::new(0, 0),
        data: ErrorData::IllegalSymbol('&'),
    })));

    verify_lexer(SOURCE, EXPECTED_TOKENS, EXPECTED_MATERIAL);
}

#[test]
fn lex_program() {
    static SOURCE: &'static str = r#"
        program vert { foo.bar(); }

        program frag {
            fn program keyworkds do_stuff() {
                bar.foo();
            }
        }
    "#;

    static EXPECTED_TOKENS: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Program, "program")),
        Ok((Token::Identifier, "vert")),
        Ok((Token::ProgramLiteral, " foo.bar(); ")),

        Ok((Token::Program, "program")),
        Ok((Token::Identifier, "frag")),
        Ok((Token::ProgramLiteral, "\n            fn program keyworkds do_stuff() {\n                bar.foo();\n            }\n        ")),

        Ok((Token::EndOfFile, "")),
    ];

    let EXPECTED_MATERIAL = Ok(MaterialSource {
        properties: vec![],
        programs: vec![
            ProgramSource::Vertex(" foo.bar(); ".to_string()),
            ProgramSource::Fragment("\n            fn program keyworkds do_stuff() {\n                bar.foo();\n            }\n        ".to_string()),
        ],
    });

    verify_lexer(SOURCE, EXPECTED_TOKENS, EXPECTED_MATERIAL);
}

#[test]
fn lex_program_error() {
    static SOURCE: &'static str = r#"
        program vert { foo.bar(); }

        program frag {
            fn program keyworkds do_stuff() {
                bar.foo();
            }
    "#;

    static EXPECTED_TOKENS: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Program, "program")),
        Ok((Token::Identifier, "vert")),
        Ok((Token::ProgramLiteral, " foo.bar(); ")),

        Ok((Token::Program, "program")),
        Ok((Token::Identifier, "frag")),

        Err((ErrorData::UnclosedProgramLiteral, "{\n            fn program keyworkds do_stuff() {\n                bar.foo();\n            }\n    ")),
    ];

    let EXPECTED_MATERIAL = Err(MaterialSourceError::ParseError(ParseError::TokenError(TokenError {
        span: Span::new(0, 0),
        data: ErrorData::UnclosedProgramLiteral,
    })));

    verify_lexer(SOURCE, EXPECTED_TOKENS, EXPECTED_MATERIAL);
}
