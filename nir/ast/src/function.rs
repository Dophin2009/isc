use crate::keywords::{self, Arrow, Colon, Comma, LParen, RParen};
use crate::punctuated::Punctuated;
use crate::{Block, Ident, Span, Spannable, Spanned, Type, Visibility};

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Punctuated<FunctionParam, Comma>,
    pub return_type: Type,
    pub body: Block,

    pub fn_t: Spanned<keywords::Function>,
    pub lparen_t: Spanned<LParen>,
    pub rparen_t: Spanned<RParen>,
    pub arrow_t: Option<Spanned<Arrow>>,
}

impl Function {
    pub fn params(&self) -> &Vec<FunctionParam> {
        &self.params.items
    }
}

impl Spannable for Function {
    #[inline]
    fn span(&self) -> crate::Span {
        Span::new(self.vis.span().start, self.body.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParam {
    pub name: Ident,
    pub ty: Type,

    pub colon_t: Spanned<Colon>,
}

impl Spannable for FunctionParam {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.name.span().start, self.ty.span().end)
    }
}
