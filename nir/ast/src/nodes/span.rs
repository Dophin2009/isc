#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

pub trait Spannable {
    fn span(&self) -> Span;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Spanned<T> {
    #[inline]
    pub fn new(inner: T, span: Span) -> Self {
        Self(inner, span)
    }

    #[inline]
    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> Spannable for Spanned<T> {
    #[inline]
    fn span(&self) -> Span {
        self.1.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
