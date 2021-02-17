use crate::keywords::{LBracket, RBracket};
use crate::{Ident, Span, Spannable, Spanned};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<ArrayType>),
    Declared(DeclaredType),
}

impl Spannable for Type {
    fn span(&self) -> Span {
        match self {
            Self::Primitive(ty) => ty.span(),
            Self::Array(ty) => ty.span(),
            Self::Declared(ty) => ty.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveType {
    pub kind: PrimitiveTypeKind,
    pub span: Span,
}

impl Spannable for PrimitiveType {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredType {
    pub name: Ident,
}

impl Spannable for DeclaredType {
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveTypeKind {
    Unit,
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayType {
    pub ty: Type,

    pub lbracket_t: Spanned<LBracket>,
    pub rbracket_t: Spanned<RBracket>,
}

impl Spannable for ArrayType {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.lbracket_t.span().start, self.rbracket_t.span().end)
    }
}
