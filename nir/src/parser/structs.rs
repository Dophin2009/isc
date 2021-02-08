use super::error::{ExpectedToken, ParseError};
use super::{Parse, ParseInput, ParseResult, Symbol};
use crate::ast::{Struct, StructField, StructFunction, Type};
use crate::token::{self, Reserved, Token};

impl<I> Parse<I> for Struct
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;

        // Ensure next token is struct.
        input.consume::<token::Struct>()?;

        // Parse struct name.
        let name = input.parse()?;

        // Ensure next token is opening brace.
        input.consume::<token::LBrace>()?;

        // Parse fields and methods.
        let fields = Vec::new();
        let functions = Vec::new();
        while !input.peek_is(&reserved!(RBrace)) {
            // Parse member visibility and patch later.
            let member_vis = input.parse()?;

            // Ensure next token is not also a visibility one.
            match input.peek() {
                // If so, actually consume that token and return an error.
                Some(peeked) if peeked.0 == reserved!(Pub) => {
                    let next = input.next().unwrap();
                    input.error(unexpectedtoken!(
                        next.1,
                        next.0,
                        ereserved!(Function),
                        ExpectedToken::Ident
                    ));
                    return Err(());
                }
                None => {
                    input.error(unexpectedeof!(ereserved!(Function), ExpectedToken::Ident));
                    return Err(());
                }
                _ => {}
            };

            // Check if member is a function or a field.
            if input.peek_is(&reserved!(Function)) {
                // If fn token next, parse as function.
                let mut function: StructFunction = input.parse()?;
                function.vis = member_vis;
                functions.push(function);
            } else {
                // Otherwise, parse as struct field.
                let mut field: StructField = input.parse()?;
                field.vis = member_vis;
                fields.push(field);
            }
        }

        input.consume::<token::RBrace>()?;

        Ok(Self {
            vis,
            name,
            fields,
            functions,
        })
    }
}

impl<I> Parse<I> for StructFunction
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;

        // Consume fn token.
        input.next_checked(&reserved!(Function), || vec![ereserved!(Function)])?;

        // Parse function name.
        let name = input.parse()?;

        // Conusme left parenthesis.
        input.consume::<token::LParen>()?;

        // Parse parameters.
        // Check if self parameter is present.
        let is_method = if input.peek_is(&Token::Ident("self".to_string())) {
            // Actually consume the self token.
            input.next();
            // Consume the comma.
            input.consume::<token::Comma>()?;
            true
        } else {
            false
        };

        let params = Vec::new();
        while !input.peek_is(&reserved!(RParen)) {
            let param = input.parse()?;

            // Consume separating comma.
            input.consume::<token::Comma>()?;
            params.push(param);
        }

        // Conusme right parenthesis.
        input.consume::<token::RParen>()?;

        let return_type = match input.consume_opt::<token::Arrow>()? {
            Some(_) => input.parse()?,
            None => Type::None,
        };

        // Parse block.
        let body = input.parse()?;

        Ok(Self {
            vis,
            name,
            params,
            return_type,
            is_method,
            body,
        })
    }
}
