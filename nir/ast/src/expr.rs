use crate::keywords::{Comma, LParen, RParen};
use crate::punctuated::Punctuated;
use crate::{Ident, Span, Spannable, Spanned};

pub use lexer::Literal;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Var(Ident),
    Literal(Literal),
    FunctionCall(FunctionCall),
    BinOp(Box<BinOpExpr>),
    UnaryOp(Box<UnaryOpExpr>),
    ArrayIndex(Box<ArrayIndex>),
}

impl Spannable for Expr {
    fn span(&self) -> Span {
        Span::new(0, 0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Ident,
    pub args: Punctuated<Expr, Comma>,

    pub lparen_t: Spanned<LParen>,
    pub rparen_t: Spanned<RParen>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinOpExpr {
    pub op: BinOp,
    pub e1: Expr,
    pub e2: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub operand: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Negative,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayIndex {
    pub array: Expr,
    pub index: Expr,
}
