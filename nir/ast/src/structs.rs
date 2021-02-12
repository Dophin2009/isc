use crate::{Block, FunctionParam, Ident, Type, Visibility};

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Vec<StructField>,
    // pub functions: Vec<StructFunction>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructField {
    pub vis: Visibility,
    pub name: Ident,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructFunction {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub is_method: bool,
    pub body: Block,
}
