// lexer! creates a struct with visibility (#struct_visibility) and name (#struct_name). It defines
// the method #struct_name::#fn_name (e.g. Lexer::stream) to return an iterator for tokens
// (LexerStream<#token_type>) parsed from the given input. See below example. On error (such as
// where no tokens can be produced from the remaining non-empty input), the error variant
// (#error_variant) is returned.
//
// Define the regular expression and their corresponding actions, highest precedence first.  See
// `regexp2` crate for supported regular expression syntax. The action expressions must return
// Option<#token_type>.
//
//
// FORMAT:
//
// #struct_visibility struct #struct_name;
// #fn_visibility fn #fn_name;
// (#span_var) -> #token_type, #error_variant;
//
//
// GENERATED:
//
// #struct_visibility struct #struct_name { ... }
//
// impl #struct_name {
//     #struct_visibility fn stream(&self, input: &str) -> Option<LexerItem<#token_type>> {
//         ...
//     }
// }

use std::fmt;

use llex::lexer;

// The type returned from the generated lexer function.
#[derive(Debug, Clone)]
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

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::Integer(i) => write!(f, "{}", i),
            Token::Float(n) => write!(f, "{}", n),
            Token::KeywordPub => write!(f, "pub"),
            Token::KeywordFn => write!(f, "fn"),
            Token::KeywordEnum => write!(f, "enum"),
            Token::LeftParenthesis => write!(f, "("),
            Token::RightParenthesis => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Error => write!(f, "<error>"),
        }
    }
}

lexer! {
    pub struct Lexer;
    // Generated:
    //
    //     pub struct Lexer { ... }
    //

    pub fn stream;
    (text) -> Token, Token::Error;
    // Generated:
    //
    //     impl struct Lexer {
    //         pub fn stream(&self, input: &str) -> LexerStream<Token, ...> {
    //             ...
    //         }
    //     }
    //


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
    r"[A-Za-z_][A-Za-z0-9_]*" => Some(Token::Ident(text.to_string())),
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
const INPUT_STR: &str = r"
pub enum Token {
    Ident(String),
    Integer(i64),
    Float(f64),
}
";

fn main() {
    let lexer = Lexer::new();
    let chars = INPUT_STR.chars();
    let tokens = lexer.stream(chars);

    for t in tokens {
        print!("('{}' {}:{}) ", t.token, t.m.start, t.m.end - 1);
    }
    println!()
}
