#![feature(trace_macros)]
// trace_macros!(true);

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
    // name of the token span. The actual function #name::advance returns an `Result` of a tuple of the token
    // type and the remaining input string, and the error type..
    //
    // Format:
    // #visibility #name(#span_identifier) -> #token_type, #error_type;
    //
    // Generated:
    // #visibility struct #name { ... }
    //
    // impl #name {
    //     pub fn advance(input: &str) -> std::result::Result<(#token_type, std::string::String), #error_type> {
    //         ...
    //     }
    // }
    pub Lexer(text) -> Token, (); ();

    // Define the regular expression and their corresponding actions, highest precedence first.
    // See `regexp2` crate for supported regular expression syntax. The action expressions must
    // return Option<#token_type>.
    r"\s" => Ok(None),
    r"pub" => Ok(Some(Token::KeywordPub)),
    r"fn" => Ok(Some(Token::KeywordFn)),
    r"enum" => Ok(Some(Token::KeywordEnum)),
    r"\(" => Ok(Some(Token::LeftParenthesis)),
    r"\)" => Ok(Some(Token::RightParenthesis)),
    r"{" => Ok(Some(Token::LeftBracket)),
    r"}" => Ok(Some(Token::RightBracket)),
    r";" => Ok(Some(Token::Semicolon)),
    r"," => Ok(Some(Token::Comma)),
    r"[A-Za-z_][A-Za-z0-9_]*" => Ok(Some(Token::Ident(text))),
    // Pair that matches integers, parses them into i64, and returns Token::Integer.
    r"[0-9]+" => {
        let i = text.parse().unwrap();
        Ok(Some(Token::Integer(i)))
    }
    r"[0-9]+(\.[0-9]+)?" => {
        let f = text.parse().unwrap();
        Ok(Some(Token::Float(f)))
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
    let mut input = String::from(INPUT_STR);

    // Consume the input and return tokens until no pattern can be matched to the remaining string.
    let mut offset = 0;
    loop {
        let (token_res, remaining) = lexer.advance(&input);
        offset += input.len() - remaining.len();
        match token_res {
            Ok(token_op) => match token_op {
                Some(token) => {
                    input = remaining;
                    print!("{:?}  ", token);
                }
                None => {
                    println!("\nFinished tokenization.");
                    break;
                }
            },
            Err(_) => {
                println!("\nFailed tokenization at position {}", offset);
                break;
            }
        }
    }
}
