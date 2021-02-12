use crate::Ident;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Declared { name: Ident },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveType {
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
