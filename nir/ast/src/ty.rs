use crate::{Ident, Span, Spannable};

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Declared(DeclaredType),
}

impl Spannable for Type {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveType {
    pub kind: PrimitiveTypeKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredType {
    pub name: Ident,
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
