use super::error::{ExpectedToken, ParseError};
use super::{Parse, ParseInput, Symbol};
use crate::ast::Struct;
use crate::token::{Reserved, Token};

impl<I> Parse<I> for Struct
where
    I: Iterator<Item = Symbol>,
{
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        // Parse visibility.
        let vis = input.parse()?;

        // Ensure next token is struct.
        input.next_checked(&reserved!(Struct), || vec![ereserved!(Struct)])?;

        // Parse struct name.
        let name = input.parse()?;

        // Ensure next token is opening brace.
        input.consume_lbrace()?;

        // Parse fields and methods.
        let fields = Vec::new();
        let functions = Vec::new();
        while !input.peek_is(&Token::Reserved(Reserved::RBrace)) {
            // let vis = input.parse()?;

            if input.peek_is(&Token::Reserved(Reserved::Function)) {
                // If fn token next, parse as function.
                // let function = input.parse()?;
                // functions.push(function);
            } else {
                // Otherwise, parse as struct field.
                // let field = input.parse()?;
                // fields.push(field);
            }
        }

        input.consume_rbrace()?;

        Ok(Self {
            vis,
            name,
            fields,
            functions,
        })
    }
}
