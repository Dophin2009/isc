mod function;
mod structs;
mod ty;

pub use function::*;
pub use structs::*;
pub use ty::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Spanned<T> {
    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn span(&self) -> &Span {
        &self.1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ident {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub segs: Vec<Ident>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {}
