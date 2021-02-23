use crate::error::{ExpectedToken, ParseError};
use crate::Result;

use std::collections::BTreeSet;
use std::rc::Rc;

use ast::{Expr, Program, Spanned};
use itertools::{Itertools, MultiPeek};
use lexer::{types as ttypes, Token};

/// Result alias for return values of parse implementations.
pub(crate) type ParseResult<T> = std::result::Result<T, ()>;

/// Alias for a lexer token with span information.
pub(crate) type Symbol = Spanned<lexer::Token>;

/// Trait implemented to parse AST nodes from lexer output.
pub(crate) trait Parse<I>
where
    I: Iterator<Item = Symbol>,
    Self: Sized,
{
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self>;
}

pub(crate) trait Peek<I>
where
    I: Iterator<Item = Symbol>,
    Self: Sized,
{
    fn peek(input: &mut ParseInput<I>) -> bool;
}

#[derive(Debug)]
pub struct Parser {}

impl Parser {
    /// Create a new [`Self`].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// Parse the input tokens into a syntax tree.
    #[inline]
    pub fn parse<I>(&self, input: I) -> Result<Program>
    where
        I: Iterator<Item = Symbol>,
    {
        let mut input = ParseInput::new(input);
        let parsed = input.parse();

        if !input.errors.is_empty() {
            Err(input.errors)
        } else {
            parsed.map_err(|_| input.errors)
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseInput<I>
where
    I: Iterator<Item = Symbol>,
{
    /// Accumulated parsing errors.
    pub errors: Vec<ParseError>,

    /// Actual lexer token stream.
    inner: MultiPeek<I>,
    /// Last span position of parsed input.
    last_pos: usize,

    /// Stored expression values.
    pub(crate) expr_cache: ExprCache,
}

impl<I> ParseInput<I>
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    pub fn new(inner: I) -> Self {
        Self {
            inner: inner.multipeek(),
            errors: Vec::new(),
            last_pos: 0,
            expr_cache: ExprCache::new(),
        }
    }

    #[inline]
    pub fn error(&mut self, error: ParseError) {
        self.errors.push(error)
    }

    #[inline]
    pub fn unexpected_eof(&mut self, expected: Vec<ExpectedToken>) {
        self.errors.push(ParseError::UnexpectedEof(expected))
    }

    #[inline]
    pub fn unexpected_token(&mut self, sy: Symbol, expected: Vec<ExpectedToken>) {
        self.errors.push(ParseError::UnexpectedToken(sy, expected))
    }

    #[inline]
    pub fn last_pos(&self) -> usize {
        self.last_pos
    }

    #[inline]
    pub fn parse<T>(&mut self) -> ParseResult<T>
    where
        T: Parse<I>,
    {
        T::parse(self)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Symbol> {
        // Get next from inner iterator.
        let next = self.inner.next();

        // If Some, then update the last span position.
        if let Some(ref sy) = next {
            self.last_pos = sy.1.end;
        }
        next
    }

    #[inline]
    pub fn next_unwrap<F>(&mut self, expected: F) -> ParseResult<Symbol>
    where
        F: Fn() -> Vec<ExpectedToken>,
    {
        self.next()
            .ok_or_else(|| self.error(ParseError::UnexpectedEof(expected())))
    }

    #[inline]
    pub fn next_checked<F>(&mut self, check: &Token, expected: F) -> ParseResult<Symbol>
    where
        F: Fn() -> Vec<ExpectedToken>,
    {
        match self.next() {
            Some(sym) if sym.0 == *check => Ok(sym),
            Some(next) => {
                self.error(ParseError::UnexpectedToken(next, expected()));
                Err(())
            }
            None => {
                self.error(ParseError::UnexpectedEof(expected()));
                Err(())
            }
        }
    }

    /// Peek next item; returns cloned symbol for simplicity.
    #[inline]
    pub fn peek(&mut self) -> Option<Symbol> {
        let ret = self.inner.peek().cloned();
        self.inner.reset_peek();
        ret
    }

    #[inline]
    pub fn peek_mult(&mut self) -> Option<&Symbol> {
        self.inner.peek()
    }

    #[inline]
    pub fn reset_peek(&mut self) {
        self.inner.reset_peek();
    }

    #[inline]
    pub fn peek_is(&mut self, expected: &Token) -> bool {
        let ret = match self.peek() {
            Some(peeked) => peeked.0 == *expected,
            None => false,
        };
        self.inner.reset_peek();
        ret
    }

    #[inline]
    pub fn is_empty(&mut self) -> bool {
        let ret = self.inner.peek().is_none();
        self.inner.reset_peek();
        ret
    }

    #[inline]
    pub fn consume<R: ttypes::ReservedVariant>(&mut self) -> ParseResult<Spanned<R>> {
        self.next_checked(&Token::Reserved(R::variant()), || {
            vec![ExpectedToken::Reserved(R::variant())]
        })
        .map(|r| Spanned::new(R::new(), r.1))
    }

    #[inline]
    pub fn consume_opt<R: ttypes::ReservedVariant>(&mut self) -> ParseResult<Option<Spanned<R>>> {
        match self.peek() {
            Some(next) if next.0 == Token::Reserved(R::variant()) => {
                let next = self.next().unwrap();
                Ok(Some(Spanned::new(R::new(), next.1)))
            }
            _ => Ok(None),
        }
    }
}

pub(crate) struct Rsv<R>(R)
where
    R: ttypes::ReservedVariant;

#[allow(dead_code)]
impl<R> Rsv<R>
where
    R: ttypes::ReservedVariant,
{
    #[inline]
    pub fn new() -> Self {
        Self(R::new())
    }

    #[inline]
    pub fn into_inner(self) -> R {
        self.0
    }

    #[inline]
    pub fn inner(&self) -> &R {
        &self.0
    }
}

impl<I, R> Parse<I> for Rsv<R>
where
    I: Iterator<Item = Symbol>,
    R: ttypes::ReservedVariant,
{
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        input.consume::<R>()?;
        Ok(Self::new())
    }
}

impl<I, R> Peek<I> for Rsv<R>
where
    I: Iterator<Item = Symbol>,
    R: ttypes::ReservedVariant,
{
    fn peek(input: &mut ParseInput<I>) -> bool {
        input.peek_is(&Token::Reserved(R::variant()))
    }
}

#[derive(Debug)]
pub(crate) struct ExprCache {
    stored: BTreeSet<Rc<Expr>>,
}

impl ExprCache {
    #[inline]
    pub const fn new() -> Self {
        Self {
            stored: BTreeSet::new(),
        }
    }

    /// Gets a reference to an existing Expr value, or clones and inserts it into storage.
    #[inline]
    pub fn get_or_insert(&mut self, expr: &Expr) -> Rc<Expr> {
        match self.stored.get(expr) {
            Some(v) => v.clone(),
            None => {
                self.stored.insert(Rc::new(expr.clone()));
                self.stored.get(expr).unwrap().clone()
            }
        }
    }
}
