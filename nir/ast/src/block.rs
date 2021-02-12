use crate::{Expr, Ident};

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VarDeclaration { lhs: Ident, rhs: Expr },
    VarAssign { lhs: Ident, rhs: Expr },
    ForLoop {},
    WhileLoop { cond: Expr },
    IfOnly { cond: Expr },
    Break,
    Continue,
    Expr(Expr),
}
