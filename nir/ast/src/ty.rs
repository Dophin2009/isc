use crate::{Ident, Span, Spannable};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Declared(DeclaredType),
}

impl Spannable for Type {
    fn span(&self) -> Span {
        match self {
            Self::Primitive(ty) => ty.span(),
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
