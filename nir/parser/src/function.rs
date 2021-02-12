use crate::punctuated::Punctuated;
use crate::{Parse, ParseInput, ParseResult, Rsv, Symbol};

use ast::{Function, FunctionParam, PrimitiveType, Type};
use lexer::types as ttypes;

impl<I> Parse<I> for Function
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let vis = input.parse()?;

        // Parse fn token.
        input.consume::<ttypes::Function>()?;

        let name = input.parse()?;

        // Parse left parenthesis.
        input.consume::<ttypes::LParen>()?;

        // Parse function parameters.
        let params = input
            .parse::<Punctuated<FunctionParam, Rsv<ttypes::Comma>>>()?
            .items;

        // Parse right parenthesis.
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
            body,
        })
    }
}

impl<I> Parse<I> for FunctionParam
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let name = input.parse()?;

        // Parse colon.
        input.consume::<ttypes::Colon>()?;

        let ty = input.parse()?;

        Ok(Self { name, ty })
    }
}
