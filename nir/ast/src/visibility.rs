use crate::{Span, Spannable};

#[derive(Clone, Debug, PartialEq)]
pub struct Visibility {
    pub kind: VisibilityKind,
    pub span: Span,
}

impl Spannable for Visibility {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VisibilityKind {
    Public,
    Private,
}
