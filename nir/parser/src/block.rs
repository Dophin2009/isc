use crate::{Parse, ParseInput, ParseResult, Symbol};

use ast::Block;
use lexer::types as ttypes;

impl<I> Parse<I> for Block
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        input.consume::<ttypes::LBrace>()?;
        input.consume::<ttypes::RBrace>()?;

        Ok(Self {})
    }
}
