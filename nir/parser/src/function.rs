use crate::{Parse, ParseInput, ParseResult, Rsv, Symbol};

use ast::{
    keywords::Comma, punctuated::Punctuated, Function, FunctionParam, PrimitiveType,
    PrimitiveTypeKind, Span, Type,
};

impl<I> Parse<I> for Function
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let vis = input.parse()?;

        // Parse fn token.
        let fn_t = input.consume()?;

        // Parse name.
        let name = input.parse()?;

        // Parse left parenthesis.
        let lparen_t = input.consume()?;

        // Parse function parameters.
        let params: Punctuated<_, Rsv<Comma>> = input.parse()?;
        let seps = params
            .seps
            .into_iter()
            .map(|sep| sep.into_inner())
            .collect();
        let params = Punctuated {
            items: params.items,
            seps,
        };

        // Parse right parenthesis.
        let rparen_t = input.consume()?;

        // Parse arrow.
        let arrow_t = input.consume_opt()?;

        // Parse the return type (if any).
        let return_type = match arrow_t {
            Some(_) => input.parse()?,
            None => {
                let last_pos = input.last_pos();

                Type::Primitive(PrimitiveType {
                    kind: PrimitiveTypeKind::Unit,
                    span: Span::new(last_pos, last_pos),
                })
            }
        };

        // Parse block.
        let body = input.parse()?;

        Ok(Self {
            vis,
            name,
            params,
            return_type,
            body,
            fn_t,
            lparen_t,
            rparen_t,
            arrow_t,
        })
    }
}

impl<I> Parse<I> for FunctionParam
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            name: input.parse()?,
            colon_t: input.consume()?,
            ty: input.parse()?,
        })
    }
}
