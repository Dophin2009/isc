use crate::error::{ExpectedToken, ParseError};
use crate::{Parse, ParseInput, Symbol};

use ast::{Function, Item, Struct};
use lexer::Reserved;

impl<I> Parse<I> for Item
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        // Parse visibility and replace later.
        let vis = input.parse()?;

        // Ensure that next token is not another visibility token.
        let peeked = match input.peek() {
            Some(peeked) if peeked.0 == reserved!(Pub) => {
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
            Some(peeked) => peeked,
            None => {
                input.error(unexpectedeof!(ereserved!(Struct), ereserved!(Function)));
                return Err(());
            }
        };

        let item = match &peeked.0 {
            reserved!(Struct) => {
                let mut s: Struct = input.parse()?;
                s.vis = vis;
                Item::Struct(s)
            }
            reserved!(Function) => {
                let mut f: Function = input.parse()?;
                f.vis = vis;
                Item::Function(f)
            }
            _ => {
                let next = input.next().unwrap();
                input.error(unexpectedtoken!(
                    next.1,
                    next.0,
                    ereserved!(Pub),
                    ereserved!(Struct),
                    ereserved!(Function)
                ));
                return Err(());
            }
        };

        Ok(item)
    }
}
