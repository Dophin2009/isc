use crate::error::{CompileError, DiagnosticEmitError};

use std::io;

use diagnostic::{AsDiagnostic, AsDiagnosticFormat};
use lexer::Lexer;
use parser::{
    ast::{Program, Span, Spanned},
    Parser,
};

#[derive(Debug)]
pub struct Compiler {
    lexer: Lexer,
    parser: Parser,
}

impl Compiler {
    #[inline]
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(),
            parser: Parser::new(),
        }
    }

    #[inline]
    pub fn parse_emit<W>(
        &self,
        w: &mut W,
        input: impl IntoIterator<Item = char>,
    ) -> Result<(), DiagnosticEmitError>
    where
        W: io::Write,
    {
        match self.parse(input) {
            Ok(ast) => {
                println!("{:#?}", ast);
            }
            Err(err) => {
                self.emit_errors(w, err)?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn parse(&self, input: impl IntoIterator<Item = char>) -> Result<Program, CompileError> {
        let tokens = self
            .lexer
            .stream(input.into_iter())
            .map(|item| Spanned::new(item.token, Span::new(item.m.start, item.m.end - 1)));

        match self.parser.parse(tokens) {
            Ok(program) => Ok(program),
            Err(errors) => Err(CompileError { parse: errors }),
        }
    }

    #[inline]
    pub fn emit_errors<W>(&self, w: &mut W, error: CompileError) -> Result<(), DiagnosticEmitError>
    where
        W: io::Write,
    {
        error.as_diagnostic(w, &AsDiagnosticFormat::Rich)
    }
}

impl Default for Compiler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
