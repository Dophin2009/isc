use crate::Symbol;

use ast::Spannable;
use lexer::Reserved;

use std::fmt;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

/// Result wrapper returned by parser.
pub type Result<T> = std::result::Result<T, Vec<ParseError>>;

#[derive(Clone, Debug, thiserror::Error)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum ParseError {
    #[error("unexpected token {:?} at position {}, expected one of {:?}", .0.inner(), .0.span().start, .1)]
    UnexpectedToken(Symbol, Vec<ExpectedToken>),
    #[error("unexpected end-of-file")]
    UnexpectedEof(Vec<ExpectedToken>),
    #[error("lexer error")]
    LexerError,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum ExpectedToken {
    Ident,
    LiteralOpaque,
    Literal(LiteralKind),
    Type,
    Reserved(Reserved),
    Expr,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
            ExpectedToken::LiteralOpaque => write!(f, "<literal>"),
            ExpectedToken::Literal(kind) => match kind {
                LiteralKind::Str => write!(f, "<string>"),
                LiteralKind::Integer => write!(f, "<integer>"),
                LiteralKind::Float => write!(f, "<float>"),
                LiteralKind::Boolean => write!(f, "<bool>"),
            },
            ExpectedToken::Type => write!(f, "<identifier>"),
            ExpectedToken::Reserved(reserved) => write!(f, "{}", reserved),
            ExpectedToken::Expr => write!(f, "<expression>"),
        }
    }
}
