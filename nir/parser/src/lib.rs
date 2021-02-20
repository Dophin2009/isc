/// Internal utility macros
#[macro_use]
#[allow(unused_macros)]
mod macros;

pub mod error;
pub mod parser;

// Parse implementations on AST nodes.
mod block;
mod expr;
mod function;
mod ident;
mod item;
mod program;
mod punctuated;
mod structs;
mod ty;
mod visibility;

/// Re-export of ast crate.
pub use ast;

// Export error facilities directly.
pub use self::error::{ExpectedToken, ParseError, Result};
// Export parser directly.
pub use self::parser::Parser;

// Internal crate usage convenience.
pub(crate) use self::parser::{Parse, ParseInput, ParseResult, Peek, Rsv, Symbol};
