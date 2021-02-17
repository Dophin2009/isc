use crate::{Span, Spannable, Spanned};

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Ident {
    pub name: Spanned<String>,
}

impl Spannable for Ident {
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Path {
    pub segments: Vec<Ident>,
}

impl Spannable for Path {
    fn span(&self) -> Span {
        let (start, end) = self
            .segments
            .first()
            .map(|item| item.span().start)
            .map(|start| (start, self.segments.last().unwrap().span().end))
            .unwrap_or_else(|| (0, 0));
        Span::new(start, end)
    }
}
