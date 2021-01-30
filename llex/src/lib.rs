#![deny(rust_2018_idioms)]
#![deny(future_incompatible)]

pub mod stream;

pub use llex_macro::lexer;
pub use stream::{LexerItem, LexerStream};

pub use regexp2;
