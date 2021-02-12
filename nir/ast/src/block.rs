use crate::{Expr, Ident, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VarDeclaration(VarDeclaration),
    VarAssign(VarAssign),
    ForLoop(ForLoop),
    WhileLoop(WhileLoop),
    IfOnly(IfOnly),
    Break(Break),
    Continue(Continue),
    Expr(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDeclaration {
    pub lhs: Ident,
    pub ty: Type,
    pub rhs: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarAssign {
    pub lhs: Ident,
    pub rhs: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    pub ident: Ident,
    pub range: Expr,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileLoop {
    pub cond: Expr,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Break;

#[derive(Clone, Debug, PartialEq)]
pub struct Continue;

#[derive(Clone, Debug, PartialEq)]
pub struct IfOnly {
    pub cond: Expr,
    pub body: Block,
}
