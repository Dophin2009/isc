#![feature(trace_macros)]
trace_macros!(true);

use anyhow::Result;
use llex::lexer;

// The type returned from the generated lexer function.
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
    // Define the name and visibility of the lexer function, as well as the type of the token and
    // name of the token span. The actual function returns an `Option` of a tuple of the token
    // type and the remaining input string.
    //
    // Format:
    // #visibility fn #function_name(#span_identifier) -> #token_type;
    //
    // Actual function signature:
    // #visibility fn #function_name(#span_identifier: &str) -> std::option::Option<#token_type, std::string::String>
    pub Lexerr(text) -> Token;

    // Define the regular expression and their corresponding actions, highest precedence first.
    // See `regexp2` crate for supported regular expression syntax. The action expressions must
    // return Option<#token_type>.
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
    // Pair that matches integers, parses them into i64, and returns Token::Integer.
    r"[0-9]+" => {
        let i = text.parse().unwrap();
        Some(Token::Integer(i))
    }
    r"[0-9]+(\.[0-9]+)?" => {
        let f = text.parse().unwrap();
        Some(Token::Float(f))
    }
}

// The input string to pass into the lexer function.
const INPUT_STR: &str = "
pub enum Token {
    Ident(String),
    Integer(i64),
    Float(f64),
}
";

fn main() -> Result<()> {
    let mut input = String::from(INPUT_STR);
    let lexer = Lexerr::new();
    // Consume the input and return tokens until no pattern can be matched to the remaining string.
    while let Some(token_t) = lexer.advance(&input) {
        input = token_t.1;
        let token = token_t.0;
        print!("{:?}  ", token);
    }

    Ok(())
}
