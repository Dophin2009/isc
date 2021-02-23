use super::keywords::DoubleColon;
use super::punctuated::Punctuated;
use super::{Span, Spannable, Spanned};

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Ident {
    pub name: Spanned<String>,
}

impl Ident {
    #[inline]
    pub fn name_str(&self) -> &str {
        self.name.0.as_ref()
    }
}

impl Spannable for Ident {
    #[inline]
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Path {
    pub segments: Punctuated<Ident, DoubleColon>,
}

impl Spannable for Path {
    #[inline]
    fn span(&self) -> Span {
        let (start, end) = self
            .segments
            .items
            .first()
            .map(|item| item.span().start)
            .map(|start| (start, self.segments.items.last().unwrap().span().end))
            .unwrap_or_else(|| (0, 0));
        Span::new(start, end)
    }
}
