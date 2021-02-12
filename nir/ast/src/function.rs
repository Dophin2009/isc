pub use crate::{Body, Ident, Type, Visibility};

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParam {
    pub name: Ident,
    pub ty: Type,
}
