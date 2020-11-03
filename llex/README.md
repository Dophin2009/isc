# Llex

Llex (lame lexer analyser generator) is an attempt at a simple lexer
generator. It uses the [`automata`](../automata) and
[`regexp2`](../regexp2) crates and takes the form of a procedural macro.

## Usage

See the [examples](./examples).

``` rust
// lexer! defines two structs with visibility (#struct_visibility) and names (#struct_name) and
// (#internal_struct_name). It defines the method #struct_name::#fn_name (e.g. Lexer::stream) to
// return an iterator for tokens (LexerStream<#token_type>) parsed from the given input. See
// below example. On error (such as where no tokens can be produced from the remaining non-empty
// input), the error variant (#error_variant) is returned.
//
// Define the regular expression and their corresponding actions, highest precedence first.
// See `regexp2` crate for supported regular expression syntax. The action expressions must
// return Option<#token_type>.
//
//
// FORMAT:
//
// #struct_visibility struct #struct_name, #internal_struct_name;
// #fn_visibility fn #fn_name;
// (#span_var) -> #token_type, #error_variant;
//
//
// GENERATED:
//
// #struct_visibility struct #struct_name { ... }
//
// #struct_visibility struct #internal_struct_name { ... }
//
// impl #struct_name {
//     #struct_visibility fn stream(&self, input: &str) -> Option<LexerItem<#token_type>> {
//         ...
//     }
// }

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

lexer! {
    pub struct Lexer, LexerInternal;
    // Generated:
    //
    //     pub struct Lexer { ... }
    //     pub struct LexerInternal { ... }
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
    let mut tokens = lexer.stream(INPUT_STR);

    while let Some(t) = tokens.next() {
        print!("{:?} ", t.token);
    }
}
```
