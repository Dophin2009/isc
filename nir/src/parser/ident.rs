use super::error::{ExpectedToken, ParseError};
use super::{Parse, ParseInput, Symbol};
use crate::ast::Ident;
use crate::token::{Reserved, Token};

impl<I> Parse<I> for Ident
where
    I: Iterator<Item = Symbol>,
{
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        let next = input.next_unwrap(|| vec![ExpectedToken::Ident])?;
        let name = match next.0 {
            Token::Ident(ident) => ident,
            _ => return Err(input.error(unexpectedtoken!(next.1, next.0, ExpectedToken::Ident))),
        };
        Ok(Self { name })
    }
}
