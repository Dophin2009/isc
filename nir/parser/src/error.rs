use ast::Span;
use lexer::{Reserved, Token};

use std::fmt;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {:?} at position {}, expected one of {:?}", .1, .0.start + 1, .2)]
    UnexpectedToken(Span, Token, Vec<ExpectedToken>),
    #[error("unexpected end-of-file")]
    UnexpectedEof(Vec<ExpectedToken>),
    #[error("lexer error")]
    LexerError,
}

#[derive(Debug, Clone)]
pub enum ExpectedToken {
    Ident,
    Literal(LiteralKind),
    Type,
    Reserved(Reserved),
}

#[derive(Debug, Clone)]
pub enum LiteralKind {
    Str,
    Integer,
    Float,
    Boolean,
}

impl fmt::Display for ExpectedToken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedToken::Ident => write!(f, "<identifier>"),
            ExpectedToken::Literal(kind) => match kind {
                LiteralKind::Str => write!(f, "<string>"),
                LiteralKind::Integer => write!(f, "<integer>"),
                LiteralKind::Float => write!(f, "<float>"),
                LiteralKind::Boolean => write!(f, "<bool>"),
            },
            ExpectedToken::Type => write!(f, "<identifier>"),
            ExpectedToken::Reserved(reserved) => write!(f, "{}", reserved),
        }
    }
}

macro_rules! unexpectedeof{
    ($($expected:expr),*) => {
        ParseError::UnexpectedEof(vec![$($expected),*])
    };
}

macro_rules! unexpectedtoken {
    ($span:expr, $token:expr, $($expected:expr),*) => {
        ParseError::UnexpectedToken($span, $token, vec![$($expected),*])
    };
}

macro_rules! ereserved {
    ($variant:ident) => {
        ExpectedToken::Reserved(Reserved::$variant)
    };
}

macro_rules! eliteral {
    ($variant:ident) => {
        ExpectedToken::Literal(Reserved::$variant)
    };
}
