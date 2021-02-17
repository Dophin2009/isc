mod block;
mod expr;
mod function;
mod ident;
mod program;
mod structs;
mod ty;
mod visibility;

pub mod punctuated;

pub use block::*;
pub use expr::*;
pub use function::*;
pub use ident::*;
pub use program::*;
pub use structs::*;
pub use ty::*;
pub use visibility::*;

pub use lexer::types as keywords;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

pub trait Spannable {
    fn span(&self) -> Span;
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self(inner, span)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> Spannable for Spanned<T> {
    fn span(&self) -> Span {
        self.1.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
