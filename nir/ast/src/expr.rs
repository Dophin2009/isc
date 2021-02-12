use crate::{Span, Spannable};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {}

impl Spannable for Expr {
    fn span(&self) -> Span {
        Span::new(0, 0)
    }
}
