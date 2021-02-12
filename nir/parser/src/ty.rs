use crate::error::{ExpectedToken, ParseError};
use crate::{Parse, ParseInput, ParseResult, Symbol};

use ast::{Ident, PrimitiveType, Type};
use lexer::{Reserved, Token};

impl<I> Parse<I> for Type
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let next = input.next_unwrap(|| {
            vec![
                ExpectedToken::Type,
                ExpectedToken::Ident,
                ereserved!(LParen),
            ]
        })?;

        let ty = match next.0 {
            Token::Type(ty) => Self::Primitive(from_lexer_type(ty)),
            Token::Ident(name) => Self::Declared {
                name: Ident { name },
            },
            Token::Reserved(Reserved::LParen) => Self::Primitive(PrimitiveType::Unit),
            _ => {
                input.error(unexpectedtoken!(
                    next.1,
                    next.0,
                    ExpectedToken::Type,
                    ExpectedToken::Ident,
                    ereserved!(LParen)
                ));
                return Err(());
            }
        };

        Ok(ty)
    }
}

fn from_lexer_type(ty: lexer::Type) -> PrimitiveType {
    match ty {
        lexer::Type::Bool => PrimitiveType::Bool,
        lexer::Type::Char => PrimitiveType::Char,
        lexer::Type::I8 => PrimitiveType::I8,
        lexer::Type::I16 => PrimitiveType::I16,
        lexer::Type::I32 => PrimitiveType::I32,
        lexer::Type::I64 => PrimitiveType::I64,
        lexer::Type::I128 => PrimitiveType::I128,
        lexer::Type::U8 => PrimitiveType::U8,
        lexer::Type::U16 => PrimitiveType::U16,
        lexer::Type::U32 => PrimitiveType::U32,
        lexer::Type::U64 => PrimitiveType::U64,
        lexer::Type::F32 => PrimitiveType::F32,
        lexer::Type::F64 => PrimitiveType::F64,
    }
}
