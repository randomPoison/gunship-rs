extern crate polygon_material as material;

use material::token::*;

#[test]
fn lex_properties() {
    static SOURCE: &'static str = r#"
        property surface_color : Color ;
        property another_thing: f32;
        property some_vec:Vector3;
    "#;
    static TOKENS: &'static [Token] = &[
        Token::Keyword(Keyword::Property),
        Token::Identifier,
        Token::Colon,
        Token::Identifier,
        Token::SemiColon,

        Token::Keyword(Keyword::Property),
        Token::Identifier,
        Token::Colon,
        Token::Identifier,
        Token::SemiColon,

        Token::Keyword(Keyword::Property),
        Token::Identifier,
        Token::Colon,
        Token::Identifier,
        Token::SemiColon,

        Token::EndOfFile,
    ];

    let mut lexer = material::lexer::Lexer::new(SOURCE);
    let mut tokens = TOKENS.iter();
    loop {
        let (token, span) = lexer.next().unwrap();
        println!("{:?}: {}", span, &SOURCE[span.begin .. span.end]);

        assert_eq!(token, *tokens.next().unwrap());

        if token == Token::EndOfFile { break; }
    }
}

#[test]
fn lex_programs() {
    static SOURCE: &'static str = r#"
        program vert { foo.bar(); }

        program frag {
            fn program keyworkds do_shit() {
                bar.foo();
            }
        }
    "#;

    static TOKENS: &'static [Token] = &[
        Token::Keyword(Keyword::Program),
        Token::Identifier,
        Token::Program,

        Token::Keyword(Keyword::Program),
        Token::Identifier,
        Token::Program,

        Token::EndOfFile,
    ];

    let mut lexer = material::lexer::Lexer::new(SOURCE);
    let mut tokens = TOKENS.iter();
    loop {
        let (token, span) = lexer.next().unwrap();
        println!("{:?}: {}", span, &SOURCE[span.begin .. span.end]);

        assert_eq!(token, *tokens.next().unwrap());

        if token == Token::EndOfFile { break; }
    }
}
