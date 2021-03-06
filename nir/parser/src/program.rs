use crate::{Parse, ParseInput, ParseResult, Symbol};

use ast::Program;

impl<I> Parse<I> for Program
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse items.
        let mut items = Vec::new();
        while input.peek().is_some() {
            let item = input.parse()?;
            items.push(item);
        }

        Ok(Self { items })
    }
}
