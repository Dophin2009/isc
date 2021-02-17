use lexer::Lexer;
use parser::{
    ast::{Program, Span, Spanned},
    ParseError, Parser,
};

use std::fmt;

#[derive(Debug)]
pub struct Compiler {
    lexer: Lexer,
    parser: Parser,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(),
            parser: Parser::new(),
        }
    }

    pub fn parse_emit(&self, input: impl IntoIterator<Item = char>) {
        match self.parse(input) {
            Ok(ast) => {
                println!("{:#?}", ast);
            }
            Err(err) => {
                self.emit_errors(vec![err]);
            }
        }
    }

    pub fn parse(&self, input: impl IntoIterator<Item = char>) -> Result<Program, CompileError> {
        let tokens = self
            .lexer
            .stream(input.into_iter())
            .map(|item| Spanned::new(item.token, Span::new(item.m.start, item.m.end - 1)));

        Ok(self.parser.parse(tokens)?)
    }

    pub fn emit_errors(&self, errors: Vec<CompileError>) {
        for error in errors {
            match error {
                CompileError::ParseError(errs) => {
                    for err in errs {
                        println!("{}", err);
                    }
                }
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    ParseError(Vec<ParseError>),
}

impl From<Vec<ParseError>> for CompileError {
    fn from(errors: Vec<ParseError>) -> Self {
        Self::ParseError(errors)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO: actual error printing
            CompileError::ParseError(errors) => write!(f, "{:#?}", errors),
        }
    }
}
