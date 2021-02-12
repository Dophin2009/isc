/// Internal utility macros
#[macro_use]
mod macros;

#[macro_use]
pub mod error;

pub mod parser;
pub mod punctuated;

// Parse implementations on AST nodes.
mod block;
mod function;
mod ident;
mod item;
mod program;
mod structs;
mod ty;
mod visibility;

/// Re-export of ast crate.
pub use ast;
pub use ast::{Span, Spanned};

pub use self::error::{ExpectedToken, ParseError, Result};
pub use self::parser::Parser;

// Reexport for internal crate usage.
pub(crate) use self::parser::{Parse, ParseInput, ParseResult, Rsv};

use lexer::Token;
pub(crate) type Symbol = Spanned<Token>;
