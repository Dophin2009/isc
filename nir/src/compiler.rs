use crate::error::CompileError;

use std::io;

use diagnostic::{AsDiagnostic, DiagnosticEmitError, DiagnosticFormat};
use irgen::Codegen;
use lexer::Lexer;
use parser::{
    ast::{Span, Spanned},
    Parser,
};

#[derive(Debug)]
pub struct Compiler {
    lexer: Lexer,
    parser: Parser,
    codegen: Codegen,
}

impl Compiler {
    #[inline]
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(),
            parser: Parser::new(),
            codegen: Codegen::new(),
        }
    }

    #[inline]
    pub fn compile(&self, input: impl IntoIterator<Item = char>) {
        let parsed = self.parse(input)?;
        self.codegen.emit(parsed)
    }

    #[inline]
    pub fn parse_emit<W>(
        &self,
        w: &mut W,
        input: impl IntoIterator<Item = char>,
    ) -> Result<ast::Program, DiagnosticEmitError>
    where
        W: io::Write,
    {
        self.parse(input).map_err(|err| self.emit_errors(w, err)?)
    }

    #[inline]
    pub fn parse(
        &self,
        input: impl IntoIterator<Item = char>,
    ) -> Result<ast::Program, CompileError> {
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
        CompileError::emit_diagnostic(&error, w, &DiagnosticFormat::Rich)
    }
}

impl Default for Compiler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
