/// Internal utility macros
#[macro_use]
mod macros;

pub mod error;
pub mod parser;
pub mod punctuated;
pub mod span;

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

// Export error facilities directly.
pub use self::error::{ExpectedToken, ParseError, Result};
// Export parser directly.
pub use self::parser::Parser;

// Internal crate usage convenience.
pub(crate) use self::parser::{Parse, ParseInput, ParseResult, Rsv, Symbol};