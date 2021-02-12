use crate::error::{ExpectedToken, ParseError};
use crate::{Parse, ParseInput, ParseResult, Rsv, Separated, Symbol};

use ast::{FunctionParam, PrimitiveType, Struct, StructField, StructFunction, Type};
use lexer::{types as ttypes, Reserved, Token};

impl<I> Parse<I> for Struct
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;

        // Ensure next token is struct.
        input.consume::<ttypes::Struct>()?;

        // Parse struct name.
        let name = input.parse()?;

        // Ensure next token is opening brace.
        input.consume::<ttypes::LBrace>()?;

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

        input.consume::<ttypes::RBrace>()?;

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
        input.consume::<ttypes::LParen>()?;

        // Parse parameters.
        // Check if self parameter is present.
        let is_method = if input.peek_is(&Token::Ident("self".to_string())) {
            // Actually consume the self token.
            input.next();
            // Consume the comma.
            input.consume::<ttypes::Comma>()?;
            true
        } else {
            false
        };

        let params = input
            .parse::<Separated<FunctionParam, Rsv<ttypes::Comma>>>()?
            .items;

        // Conusme right parenthesis.
        input.consume::<ttypes::RParen>()?;

        // Parse the return type (if any).
        let return_type = match input.consume_opt::<ttypes::Arrow>()? {
            Some(_) => input.parse()?,
            None => Type::Primitive(PrimitiveType::Unit),
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
