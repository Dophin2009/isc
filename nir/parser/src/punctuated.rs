use crate::{Parse, ParseInput, ParseResult, Symbol};

#[derive(Debug, Clone)]
pub struct Punctuated<T, S> {
    pub items: Vec<T>,
    pub seps: Vec<S>,
}

impl<I, T, S> Parse<I> for Punctuated<T, S>
where
    I: Iterator<Item = Symbol>,
    T: Parse<I>,
    S: Parse<I>,
{
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Try to parse first item. If EOF or other token encountered, return empty result.
        let try_parsed = input.parse().ok();
        let first_item = match try_parsed {
            Some(t) => t,
            None => {
                return Ok(Self {
                    items: vec![],
                    seps: vec![],
                })
            }
        };

        let mut items = Vec::new();
        let mut seps = Vec::new();
        items.push(first_item);

        loop {
            let sep = match input.parse().ok() {
                Some(s) => s,
                None => break,
            };

            let item = input.parse()?;
            items.push(item);
            seps.push(sep);
        }

        Ok(Self { items, seps })
    }
}
