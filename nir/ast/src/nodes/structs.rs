use super::keywords::{self, Colon, Comma, LBrace, RBrace};
use super::punctuated::Punctuated;
use super::{Ident, Span, Spannable, Spanned, Type, Visibility};

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

/// Node a struct definition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Struct {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Punctuated<StructField, Comma>,
    pub struct_t: Spanned<keywords::Struct>,
    pub lbrace_t: Spanned<LBrace>,
    pub rbrace_t: Spanned<RBrace>,
}

impl Struct {
    #[inline]
    pub fn fields(&self) -> &Vec<StructField> {
        &self.fields.items
    }
}

impl Spannable for Struct {
    #[inline]
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
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.vis.span().start, self.ty.span().end)
    }
}
