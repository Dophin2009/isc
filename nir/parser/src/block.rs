use crate::{ExpectedToken, Parse, ParseInput, ParseResult, Symbol};

use ast::{Block, Statement};
use lexer::types as ttypes;

impl<I> Parse<I> for Block
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse left brace.
        input.consume::<ttypes::LBrace>()?;

        // Parse statements.
        let mut statements = Vec::new();
        while !input.peek_is(&reserved!(RBrace)) {
            let statement = input.parse()?;
            statements.push(statement);
        }

        // Parse right brace.
        input.consume::<ttypes::RBrace>()?;

        Ok(Self { statements })
    }
}

impl<I> Parse<I> for Statement
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let peeked = input.peek().ok_or_else(|| {
            unexpectedeof!(
                ereserved!(Let),
                ExpectedToken::Ident,
                ereserved!(For),
                ereserved!(While),
                ereserved!(If)
            )
        });

        Ok(Self {})
    }
}
