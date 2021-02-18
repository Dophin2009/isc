use super::{Function, Span, Spannable, Struct};
use crate::Scope;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Program {
    pub items: Vec<Item>,
    pub scope: Scope,
}

impl Program {
    pub fn new() -> Self {
        Self {
            items: vec![],
            scope: Scope::new(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Spannable for Program {
    fn span(&self) -> Span {
        let (start, end) = self
            .items
            .first()
            .map(|item| item.span().start)
            .map(|start| (start, self.items.last().unwrap().span().end))
            .unwrap_or_else(|| (0, 0));
        Span::new(start, end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

impl Spannable for Item {
    fn span(&self) -> Span {
        match self {
            Self::Struct(s) => s.span(),
            Self::Function(f) => f.span(),
        }
    }
}
