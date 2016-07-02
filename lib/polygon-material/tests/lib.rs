#![feature(const_fn)]

extern crate polygon_material as material;

use material::lexer::{Error, ErrorData, Lexer};
use material::token::*;

/// Helper function for verifying the output of the lexer.
///
/// Takes a source string an array of expected outputs and verifies that the two are the same.
fn verify_lexer(source: &str, expected: &[Result<(Token, &'static str), (ErrorData, &'static str)>]) {
    let mut lexer = Lexer::new(source);

    for expected in expected {
        let actual = lexer
            .next()
            .map(|(token, span)| (token, &source[span.begin..span.end]))
            .map_err(|error| (error.data, &source[error.span.begin..error.span.end]));
        assert_eq!(*expected, actual);
    }

    assert!(lexer.is_done());
}

#[test]
fn lex_properties() {
    static SOURCE: &'static str = r#"
        property surface_color : Color ;
        property another_thing: f32;
        property some_vec:Vector3;
    "#;

    static EXPECTED: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Keyword(Keyword::Property), "property")),
        Ok((Token::Identifier, "surface_color")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "Color")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::Keyword(Keyword::Property), "property")),
        Ok((Token::Identifier, "another_thing")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "f32")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::Keyword(Keyword::Property), "property")),
        Ok((Token::Identifier, "some_vec")),
        Ok((Token::Colon, ":")),
        Ok((Token::Identifier, "Vector3")),
        Ok((Token::SemiColon, ";")),

        Ok((Token::EndOfFile, "")),
    ];

    verify_lexer(SOURCE, EXPECTED);
}

#[test]
fn lex_sybmol_error() {
    static SOURCE: &'static str = r#"
        property prog: &&
    "#;

    static EXPECTED: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Keyword(Keyword::Property), "property")),
        Ok((Token::Identifier, "prog")),
        Ok((Token::Colon, ":")),
        Err((ErrorData::IllegalSymbol('&'), "&")),
    ];

    verify_lexer(SOURCE, EXPECTED);
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

    static EXPECTED: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Keyword(Keyword::Program), "program")),
        Ok((Token::Identifier, "vert")),
        Ok((Token::Program, " foo.bar(); ")),

        Ok((Token::Keyword(Keyword::Program), "program")),
        Ok((Token::Identifier, "frag")),
        Ok((Token::Program, "\n            fn program keyworkds do_stuff() {\n                bar.foo();\n            }\n        ")),

        Ok((Token::EndOfFile, "")),
    ];

    verify_lexer(SOURCE, EXPECTED);
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

    static EXPECTED: &'static [Result<(Token, &'static str), (ErrorData, &'static str)>] = &[
        Ok((Token::Keyword(Keyword::Program), "program")),
        Ok((Token::Identifier, "vert")),
        Ok((Token::Program, " foo.bar(); ")),

        Ok((Token::Keyword(Keyword::Program), "program")),
        Ok((Token::Identifier, "frag")),

        Err((ErrorData::UnclosedProgramLiteral, "{\n            fn program keyworkds do_stuff() {\n                bar.foo();\n            }\n    ")),
    ];

    verify_lexer(SOURCE, EXPECTED);
}
