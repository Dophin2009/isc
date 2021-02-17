use crate::keywords::{Comma, LBracket, LParen, RBracket, RParen};
use crate::punctuated::Punctuated;
use crate::{Ident, Path, Span, Spannable, Spanned};

pub use lexer::Literal;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum Expr {
    Var(Ident),
    Literal(Spanned<Literal>),
    ArrayLiteral(Box<ArrayLiteral>),
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
            Self::ArrayLiteral(v) => v.span(),
            Self::FunctionCall(v) => v.span(),
            Self::BinOp(v) => v.span(),
            Self::UnaryOp(v) => v.span(),
            Self::ArrayIndex(v) => v.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct ArrayLiteral {
    pub elements: Punctuated<Expr, Comma>,

    pub lbracket_t: Spanned<LBracket>,
    pub rbracket_t: Spanned<RBracket>,
}

impl Spannable for ArrayLiteral {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.lbracket_t.span().start, self.rbracket_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct FunctionCall {
    pub function: Path,
    pub args: Punctuated<Expr, Comma>,

    pub lparen_t: Spanned<LParen>,
    pub rparen_t: Spanned<RParen>,
}

impl Spannable for FunctionCall {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.function.span().start, self.rparen_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equ,
    Nequ,
    GtEqu,
    Gt,
    LtEqu,
    Lt,

    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum UnaryOp {
    Negative,
    /// Boolean negation
    Not,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
