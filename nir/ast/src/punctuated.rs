use crate::{Span, Spannable};

#[derive(Debug, Clone, PartialEq)]
pub struct Punctuated<T, S> {
    pub items: Vec<T>,
    pub seps: Vec<S>,
}

impl<T, S> Punctuated<T, S> {
    #[inline]
    pub fn new(items: Vec<T>, seps: Vec<S>) -> Self {
        Self { items, seps }
    }
}

impl<T, S> Default for Punctuated<T, S> {
    #[inline]
    fn default() -> Self {
        Self {
            items: vec![],
            seps: vec![],
        }
    }
}

impl<T, S> Spannable for Punctuated<T, S>
where
    T: Spannable,
    S: Spannable,
{
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
