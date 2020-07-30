#![feature(trace_macros)]
// trace_macros!(true);

use anyhow::Result;
use lexgen::lexer;

#[derive(Debug)]
pub enum Token {
    Ident(String),
    Integer(i64),
    Float(f64),

    KeywordPub,
    KeywordFn,
    KeywordEnum,

    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
}

lexer! {
    pub fn next_token(text) -> Token;

    r"\s" => None,
    r"pub" => Some(Token::KeywordPub),
    r"fn" => Some(Token::KeywordFn),
    r"enum" => Some(Token::KeywordEnum),
    r"\(" => Some(Token::LeftParenthesis),
    r"\)" => Some(Token::RightParenthesis),
    r"{" => Some(Token::LeftBracket),
    r"}" => Some(Token::RightBracket),
    r";" => Some(Token::Semicolon),
    r"," => Some(Token::Comma),
    r"[A-Za-z_][A-Za-z0-9_]*" => Some(Token::Ident(text)),
    r"[0-9]+" => {
        let i = text.parse().unwrap();
        Some(Token::Integer(i))
    }
    r"[0-9]+(\.[0-9]+)?" => {
        let f = text.parse().unwrap();
        Some(Token::Float(f))
    }
}

const INPUT_STR: &str = "
pub enum Token {
    Ident(String),
    Integer(i64),
    Float(f64),
}
";

fn main() -> Result<()> {
    let mut input = String::from(INPUT_STR);
    while let Some(token_t) = next_token(&input) {
        input = token_t.1;
        let token = token_t.0;
        print!("{:?}  ", token);
    }

    Ok(())
}
