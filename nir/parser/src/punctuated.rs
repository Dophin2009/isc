use crate::{Parse, ParseInput, ParseResult, Peek, Symbol};

use ast::punctuated::Punctuated;

impl<I, T, S> Parse<I> for Punctuated<T, S>
where
    I: Iterator<Item = Symbol>,
    T: Parse<I>,
    S: Parse<I> + Peek<I>,
{
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let mut items = Vec::new();
        let mut seps = Vec::new();

        while !input.is_empty() {
            let item = input.parse()?;
            items.push(item);

            if !S::peek(input) {
                break;
            }

            let sep = input.parse()?;
            seps.push(sep);
        }

        Ok(Self { items, seps })
    }
}
