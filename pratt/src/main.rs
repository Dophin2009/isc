mod ast;
mod lexer;
mod parser;
mod token;

use crate::lexer::Lexer;
use crate::parser::{ParseError, Parser};

use std::io::{self, Write};

fn main() -> Result<(), Error> {
    let lexer = Lexer::new();
    let parser = Parser::new();

    let stdin = io::stdin();

    let mut buf = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;

        stdin.read_line(&mut buf)?;
        let tokens = lexer.stream(buf.chars()).map(|item| item.token);

        let ast = parser.parse(tokens)?;
        println!("{}", ast);

        buf.clear();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] io::Error),
    #[error("error during parsing")]
    ParseError(#[from] ParseError),
}
