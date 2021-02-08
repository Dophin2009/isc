#[macro_use]
mod error;

mod ident;
mod item;
mod program;
mod structs;
mod visibility;

use self::error::{ExpectedToken, ParseError};
use crate::ast::Program;
use crate::token::{Reserved, Token};

use std::iter::Peekable;

pub type ParseResult<T> = std::result::Result<T, Vec<ParseError>>;

pub trait Parse<I>
where
    I: Iterator<Item = Symbol>,
    Self: Sized,
{
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()>;
}

#[derive(Clone, Debug)]
pub struct Symbol(pub Token, pub Span);

#[derive(Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct ParseInput<I>
where
    I: Iterator<Item = Symbol>,
{
    pub inner: Peekable<I>,
    pub errors: Vec<ParseError>,
}

impl<I> ParseInput<I>
where
    I: Iterator<Item = Symbol>,
{
    pub fn new(inner: I) -> Self {
        Self {
            inner: inner.peekable(),
            errors: Vec::new(),
        }
    }

    pub fn parse<T>(&mut self) -> Result<T, ()>
    where
        T: Parse<I>,
    {
        T::parse(self)
    }

    pub fn next(&mut self) -> Option<Symbol> {
        self.inner.next()
    }

    pub fn next_unwrap<F>(&mut self, expected: F) -> Result<Symbol, ()>
    where
        F: Fn() -> Vec<ExpectedToken>,
    {
        self.next()
            .ok_or_else(|| self.error(ParseError::UnexpectedEof(expected())))
    }

    pub fn next_checked<F>(&mut self, check: &Token, expected: F) -> Result<Symbol, ()>
    where
        F: Fn() -> Vec<ExpectedToken>,
    {
        match self.next() {
            Some(sym) if sym.0 == *check => Ok(sym),
            Some(_) => Err(self.error(ParseError::UnexpectedEof(expected()))),
            None => Err(self.error(ParseError::UnexpectedEof(expected()))),
        }
    }

    pub fn peek(&mut self) -> Option<&Symbol> {
        self.inner.peek()
    }

    pub fn peek_is(&mut self, expected: &Token) -> bool {
        match self.peek() {
            Some(peeked) => peeked.0 == *expected,
            None => false,
        }
    }

    pub fn peek_unwrap<F>(&mut self, expected: F) -> Result<&Symbol, ()>
    where
        F: Fn() -> Vec<ExpectedToken>,
    {
        if let Some(peeked) = self.inner.peek() {
            Ok(peeked)
        } else {
            self.error(ParseError::UnexpectedEof(expected()));
            Err(())
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.inner.peek().is_none()
    }

    pub fn error(&mut self, error: ParseError) {
        self.errors.push(error)
    }

    pub fn consume_lbrace(&mut self) -> Result<(), ()> {
        self.next_checked(&reserved!(LBrace), || vec![ereserved!(LBrace)])?;
        Ok(())
    }

    pub fn consume_rbrace(&mut self) -> Result<(), ()> {
        self.next_checked(&reserved!(RBrace), || vec![ereserved!(LBrace)])?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Parser {}

impl Parser {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// Parse the input tokens into a syntax tree.
    #[inline]
    pub fn parse<I>(&self, input: I) -> ParseResult<Program>
    where
        I: Iterator<Item = Symbol>,
    {
        let mut input = ParseInput::new(input);
        input.parse().map_err(|_| input.errors)
    }
}
