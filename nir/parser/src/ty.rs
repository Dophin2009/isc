use crate::{ExpectedToken, Parse, ParseInput, ParseResult, Symbol};

use ast::{
    keywords::{LBracket, RBracket, RParen},
    ArrayType, DeclaredType, Ident, PrimitiveType, PrimitiveTypeKind, Span, Spanned, Type,
};
use lexer::Token;

impl<I> Parse<I> for Type
where
    I: Iterator<Item = Symbol>,
{
    // Parse a type use (not declaration).
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let next = input.next_unwrap(|| {
            vec![
                ExpectedToken::Type,
                ExpectedToken::Ident,
                ereserved!(LParen),
                ereserved!(LBracket),
            ]
        })?;

        let ty = match next.0 {
            Token::Type(ty) => from_lexer_type(ty, next.1),
            // Non-primitive type; check symbol table stack for its existence.
            Token::Ident(name) => Type::Declared(DeclaredType {
                name: Ident {
                    name: Spanned::new(name, next.1),
                },
            }),
            reserved!(LParen) => {
                input.consume::<RParen>()?;
                Type::Primitive(PrimitiveType {
                    kind: PrimitiveTypeKind::Unit,
                    span: next.1,
                })
            }
            reserved!(LBracket) => Type::Array(Box::new(ArrayType {
                lbracket_t: Spanned::new(LBracket, next.1),
                ty: input.parse()?,
                rbracket_t: input.consume()?,
            })),
            _ => {
                input.error(unexpectedtoken!(
                    next.1,
                    next.0,
                    ExpectedToken::Type,
                    ExpectedToken::Ident,
                    ereserved!(LParen),
                    ereserved!(LBracket)
                ));
                return Err(());
            }
        };

        Ok(ty)
    }
}

#[inline]
fn from_lexer_type(ty: lexer::Type, span: Span) -> Type {
    macro_rules! primitive {
        ($variant:ident) => {
            Type::Primitive(PrimitiveType {
                kind: PrimitiveTypeKind::$variant,
                span,
            })
        };
    };

    match ty {
        lexer::Type::Str => Type::Array(Box::new(ArrayType {
            lbracket_t: Spanned::new(LBracket, Span::new(span.start, span.start)),
            rbracket_t: Spanned::new(RBracket, Span::new(span.end, span.end)),
            ty: Type::Primitive(PrimitiveType {
                kind: PrimitiveTypeKind::Char,
                span: Span::new(span.start, span.end),
            }),
        })),
        lexer::Type::Bool => primitive!(Bool),
        lexer::Type::Char => primitive!(Char),
        lexer::Type::I8 => primitive!(I8),
        lexer::Type::I16 => primitive!(I16),
        lexer::Type::I32 => primitive!(I32),
        lexer::Type::I64 => primitive!(I64),
        lexer::Type::I128 => primitive!(I128),
        lexer::Type::U8 => primitive!(U8),
        lexer::Type::U16 => primitive!(U16),
        lexer::Type::U32 => primitive!(U32),
        lexer::Type::U64 => primitive!(U64),
        lexer::Type::F32 => primitive!(F32),
        lexer::Type::F64 => primitive!(F64),
    }
}
