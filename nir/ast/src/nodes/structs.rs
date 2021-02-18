use super::keywords::{self, Colon, Comma, LBrace, RBrace};
use super::punctuated::Punctuated;
use super::{Block, FunctionParam, Ident, Span, Spannable, Spanned, Type, Visibility};

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Struct {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Punctuated<StructField, Comma>,
    // pub functions: Vec<StructFunction>,
    pub struct_t: Spanned<keywords::Struct>,
    pub lbrace_t: Spanned<LBrace>,
    pub rbrace_t: Spanned<RBrace>,
}

impl Struct {
    pub fn fields(&self) -> &Vec<StructField> {
        &self.fields.items
    }
}

impl Spannable for Struct {
    fn span(&self) -> Span {
        Span::new(self.struct_t.span().start, self.rbrace_t.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct StructField {
    pub vis: Visibility,
    pub name: Ident,
    pub colon_t: Spanned<Colon>,
    pub ty: Type,
}

impl Spannable for StructField {
    fn span(&self) -> Span {
        Span::new(self.vis.span().start, self.ty.span().end)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct StructFunction {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub is_method: bool,
    pub body: Block,

    pub fn_t: Spanned<keywords::Function>,
    pub lparen_t: Spanned<keywords::LParen>,
    pub rparen_t: Spanned<keywords::RParen>,
}

impl Spannable for StructFunction {
    fn span(&self) -> Span {
        Span::new(self.vis.span().start, self.body.span().end)
    }
}
