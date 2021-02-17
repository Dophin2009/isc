use crate::keywords::{self, Colon, Else, Equ, For, If, In, LBrace, Let, RBrace, Semicolon, While};
use crate::{Expr, Ident, Span, Spannable, Spanned, Type};

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Block {
    pub statements: Vec<Statement>,

    pub lbrace_t: Spanned<LBrace>,
    pub rbrace_t: Spanned<RBrace>,
}

impl Spannable for Block {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.lbrace_t.span().start, self.rbrace_t.span().start)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum Statement {
    VarDeclaration(VarDeclaration),
    VarAssign(VarAssign),
    ForLoop(ForLoop),
    WhileLoop(WhileLoop),
    IfElse(IfElse),
    Break(Break),
    Continue(Continue),
    Expr(ExprStatement),
}

impl Spannable for Statement {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::VarDeclaration(v) => v.span(),
            Self::VarAssign(v) => v.span(),
            Self::ForLoop(v) => v.span(),
            Self::WhileLoop(v) => v.span(),
            Self::IfElse(v) => v.span(),
            Self::Break(v) => v.span(),
            Self::Continue(v) => v.span(),
            Self::Expr(v) => v.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct VarDeclaration {
    pub lhs: Ident,
    pub ty: Type,
    pub rhs: Expr,

    pub let_t: Spanned<Let>,
    pub colon_t: Spanned<Colon>,
    pub equ_t: Spanned<Equ>,
    pub semicolon_t: Spanned<Semicolon>,
}

impl Spannable for VarDeclaration {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.let_t.span().start, self.semicolon_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct VarAssign {
    pub lhs: Ident,
    pub rhs: Expr,

    pub equ_t: Spanned<Equ>,
    pub semicolon_t: Spanned<Semicolon>,
}

impl Spannable for VarAssign {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.lhs.span().start, self.semicolon_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct ForLoop {
    pub ident: Ident,
    pub range: Expr,
    pub body: Block,

    pub for_t: Spanned<For>,
    pub in_t: Spanned<In>,
}

impl Spannable for ForLoop {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.for_t.span().start, self.body.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct WhileLoop {
    pub cond: Expr,
    pub body: Block,

    pub while_t: Spanned<While>,
}

impl Spannable for WhileLoop {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.while_t.span().start, self.body.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Break {
    pub break_t: Spanned<keywords::Break>,
    pub semicolon_t: Spanned<Semicolon>,
}

impl Spannable for Break {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.break_t.span().start, self.semicolon_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Continue {
    pub continue_t: Spanned<keywords::Continue>,
    pub semicolon_t: Spanned<Semicolon>,
}

impl Spannable for Continue {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.continue_t.span().start, self.semicolon_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct IfElse {
    pub head: IfBranch,
}

impl Spannable for IfElse {
    #[inline]
    fn span(&self) -> Span {
        self.head.span()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct IfBranch {
    pub cond: Expr,
    pub body: Block,
    pub else_body: Option<Box<ElseBranch>>,

    pub if_t: Spanned<If>,
}

impl Spannable for IfBranch {
    #[inline]
    fn span(&self) -> Span {
        let end = match &self.else_body {
            Some(b) => b.span().end,
            None => self.body.span().end,
        };
        Span::new(self.if_t.span().start, end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub enum ElseBranch {
    If {
        branch: IfBranch,
        else_t: Spanned<Else>,
    },
    Block {
        inner: Block,
        else_t: Spanned<Else>,
    },
}

impl Spannable for ElseBranch {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::If { branch, else_t } => Span::new(else_t.span().start, branch.span().end),
            Self::Block { inner, else_t } => Span::new(else_t.span().start, inner.span().end),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct ExprStatement {
    pub expr: Expr,

    pub semicolon_t: Spanned<Semicolon>,
}

impl Spannable for ExprStatement {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.expr.span().start, self.semicolon_t.span().end)
    }
}
