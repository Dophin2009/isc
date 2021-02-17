use crate::error::ExpectedToken;
use crate::parser::Rsv;
use crate::{Parse, ParseInput, ParseResult, Symbol};

use ast::{keywords::DoubleColon, punctuated::Punctuated, Ident, Path, Spanned};
use lexer::Token;

impl<I> Parse<I> for Ident
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let next = input.next_unwrap(|| vec![ExpectedToken::Ident])?;
        let name = match next.0 {
            Token::Ident(ident) => ident,
            _ => {
                input.error(unexpectedtoken!(next.1, next.0, ExpectedToken::Ident));
                return Err(());
            }
        };

        Ok(Self {
            name: Spanned::new(name, next.1),
        })
    }
}

impl<I> Parse<I> for Path
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let segments = input.parse::<Punctuated<_, Rsv<DoubleColon>>>()?;
        let segments = Punctuated::new(
            segments.items,
            segments.seps.into_iter().map(Rsv::into_inner).collect(),
        );
        Ok(Self { segments })
    }
}
