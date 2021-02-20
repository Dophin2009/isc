use crate::{Parse, ParseInput, Symbol};

use ast::{Function, Item, Struct};

impl<I> Parse<I> for Item
where
    I: Iterator<Item = Symbol>,
{
    /// Parse a top-level item, either a struct or function declaration.
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        // Parse visibility and replace later.
        let vis = input.parse()?;

        // Ensure that next token is not another visibility token.
        let peeked = match input.peek() {
            Some(peeked) if peeked.0 == reserved!(Pub) => {
                // If so, actually consume that token and return an error.
                let next = input.next().unwrap();
                input.unexpected_token(next, vec![ereserved!(Struct), ereserved!(Function)]);
                return Err(());
            }
            Some(peeked) => peeked,
            None => {
                input.unexpected_eof(vec![ereserved!(Struct), ereserved!(Function)]);
                return Err(());
            }
        };

        let item = match &peeked.0 {
            // Parse a struct.
            reserved!(Struct) => {
                let mut s: Struct = input.parse()?;
                // Patch visibility.
                s.vis = vis;

                Item::Struct(s)
            }
            // Parse a function.
            reserved!(Function) => {
                let mut f: Function = input.parse()?;
                // Patch visibility.
                f.vis = vis;

                Item::Function(f)
            }
            // If neither, throw an error.
            _ => {
                let next = input.next().unwrap();
                input.unexpected_token(
                    next,
                    vec![ereserved!(Pub), ereserved!(Struct), ereserved!(Function)],
                );
                return Err(());
            }
        };

        Ok(item)
    }
}
