use crate::keywords::{self, Colon, Equ, For, If, In, LBrace, Let, RBrace, Semicolon, While};
use crate::{Expr, Ident, Span, Spannable, Spanned, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub lbrace_t: Spanned<LBrace>,
    pub rbrace_t: Spanned<RBrace>,
}

impl Spannable for Block {
    #[inline]
    fn span(&self) -> Span {
        let (start, end) = self
            .statements
            .first()
            .map(|item| item.span().start)
            .map(|start| (start, self.statements.last().unwrap().span().end))
            .unwrap_or_else(|| (0, 0));
        Span::new(start, end)
    }
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

impl Spannable for Statement {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::VarDeclaration(v) => v.span(),
            Self::VarAssign(v) => v.span(),
            Self::ForLoop(v) => v.span(),
            Self::WhileLoop(v) => v.span(),
            Self::IfOnly(v) => v.span(),
            Self::Break(v) => v.span(),
            Self::Continue(v) => v.span(),
            Self::Expr(v) => v.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
pub struct IfOnly {
    pub cond: Expr,
    pub body: Block,

    pub if_t: Spanned<If>,
}

impl Spannable for IfOnly {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.if_t.span().start, self.body.span().end)
    }
}
