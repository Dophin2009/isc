mod ast;
mod lexer;
mod parser;
mod token;

use crate::lexer::Lexer;
use crate::parser::{ParseError, Parser, Span, Symbol};
use crate::token::Token;

use llex::LexerItem;

use std::io::{self, Write};

fn main() -> Result<(), Error> {
    let lexer = Lexer::new();
    let parser = Parser::new();

    let stdin = io::stdin();

    let mut buf = String::new();
    loop {
        print!("> ");
        if let Err(err) = io::stdout().flush() {
            print_error(err.into());
        }

        stdin.read_line(&mut buf)?;
        let tokens = lexer.stream(buf.chars()).map(|LexerItem { token, m }| {
            let end = if token == Token::Error {
                m.end
            } else {
                m.end - 1
            };
            Symbol(token, Span::new(m.start, end))
        });

        match parser.parse(tokens) {
            Ok(ast) => println!("{}", ast),
            Err(err) => print_error(err.into()),
        };

        buf.clear();
    }
}

fn print_error(err: Error) {
    print!("error: ");
    match err {
        Error::IoError(err) => println!("an I/O error was encountered: {}", err),
        Error::ParseError(err) => match err {
            ParseError::UnexpectedToken(span, token, expected) => {
                let expected = expected
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "unexpected {} at position {}; expected one of {}",
                    token,
                    span.start + 1,
                    expected
                );
            }
            _ => println!("{}", err),
        },
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] io::Error),
    #[error("error during parsing")]
    ParseError(#[from] ParseError),
}
