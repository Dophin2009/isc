use crate::{Parse, ParseError, ParseInput, ParseResult, Symbol};

use ast::Program;

impl<I> Parse<I> for Program
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Push root scope object.
        input.sm.push_new();

        // Parse items.
        let mut items = Vec::new();
        while input.peek().is_some() {
            let item = input.parse()?;
            items.push(item);
        }

        // Pop root scope.
        let scope = input.sm.pop().unwrap();

        // Ensure a main function is specified.
        if scope.contains("main") {
            Ok(Self { items, scope })
        } else {
            input.error(ParseError::NoMainFunction);
            Err(())
        }
    }
}
