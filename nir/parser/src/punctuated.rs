use crate::{Parse, ParseInput, ParseResult, Symbol};

use ast::punctuated::Punctuated;

impl<I, T, S> Parse<I> for Punctuated<T, S>
where
    I: Iterator<Item = Symbol>,
    T: Parse<I>,
    S: Parse<I>,
{
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let mut items = Vec::new();
        let mut seps = Vec::new();

        let first_item = input.parse()?;
        items.push(first_item);

        while let Some(sep) = input.parse().ok() {
            let item = input.parse()?;
            items.push(item);
            seps.push(sep);
        }

        Ok(Self { items, seps })
    }
}
