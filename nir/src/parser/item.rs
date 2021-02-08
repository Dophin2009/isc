use super::error::{ExpectedToken, ParseError};
use super::{Parse, ParseInput, Symbol};
use crate::ast::{Item, Struct};
use crate::token::{Reserved, Token};

impl<I> Parse<I> for Item
where
    I: Iterator<Item = Symbol>,
{
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        // Parse visibility and replace later.
        let vis = input.parse()?;

        // Ensure that next token is not another visibility token.
        let peeked = input.peek_unwrap(|| vec![ereserved!(Struct), ereserved!(Function)])?;
        if peeked.0 == reserved!(Pub) {
            // If so, actually consume that token and return an error.
            let next = input.next().unwrap();
            input.error(unexpectedtoken!(
                next.1,
                next.0,
                ereserved!(Struct),
                ereserved!(Function)
            ));
            return Err(());
        }

        let item = match &peeked.0 {
            reserved!(Struct) => {
                let mut s: Struct = input.parse()?;
                s.vis = vis;
                Item::Struct(s)
            }
            // reserved!(Function) => Item::Function(input.parse()?),
            _ => {
                input.next();
                return Err(input.error(unexpectedtoken!(
                    peeked.1,
                    peeked.0,
                    ereserved!(Pub),
                    ereserved!(Struct),
                    ereserved!(Function)
                )));
            }
        };

        Ok(item)
    }
}
