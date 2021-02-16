use crate::keywords::{Comma, LBracket, LParen, RBracket, RParen};
use crate::punctuated::Punctuated;
use crate::{Ident, Span, Spannable, Spanned};

pub use lexer::Literal;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Var(Ident),
    Literal(Spanned<Literal>),
    FunctionCall(FunctionCall),
    BinOp(Box<BinOpExpr>),
    UnaryOp(Box<UnaryOpExpr>),
    ArrayIndex(Box<ArrayIndex>),
}

impl Spannable for Expr {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Var(v) => v.span(),
            Self::Literal(v) => v.span(),
            Self::FunctionCall(v) => v.span(),
            Self::BinOp(v) => v.span(),
            Self::UnaryOp(v) => v.span(),
            Self::ArrayIndex(v) => v.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Ident,
    pub args: Punctuated<Expr, Comma>,

    pub lparen_t: Spanned<LParen>,
    pub rparen_t: Spanned<RParen>,
}

impl Spannable for FunctionCall {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.name.span().start, self.rparen_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinOpExpr {
    pub op: Spanned<BinOp>,
    pub e1: Expr,
    pub e2: Expr,
}

impl BinOpExpr {
    #[inline]
    pub fn op(&self) -> &BinOp {
        &self.op.0
    }
}

impl Spannable for BinOpExpr {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.e1.span().start, self.e2.span().end)
    }
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
    pub op: Spanned<UnaryOp>,
    pub operand: Expr,
}

impl UnaryOpExpr {
    #[inline]
    pub fn op(&self) -> &UnaryOp {
        &self.op.0
    }
}

impl Spannable for UnaryOpExpr {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.op.span().start, self.operand.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Negative,
    /// Boolean negation
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayIndex {
    pub array: Expr,
    pub index: Expr,

    pub lbracket_t: Spanned<LBracket>,
    pub rbracket_t: Spanned<RBracket>,
}

impl Spannable for ArrayIndex {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.array.span().start, self.rbracket_t.span().end)
    }
}
