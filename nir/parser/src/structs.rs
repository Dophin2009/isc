use crate::{Parse, ParseInput, ParseResult, Rsv, Symbol};

use ast::{keywords::Comma, punctuated::Punctuated, Struct, StructField};

impl<I> Parse<I> for Struct
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;

        // Ensure next token is struct.
        let struct_t = input.consume()?;

        // Parse struct name.
        let name = input.parse()?;

        // Ensure next token is opening brace.
        let lbrace_t = input.consume()?;

        // Parse fields.
        let fields = input.parse::<Punctuated<StructField, Rsv<Comma>>>()?;
        let seps = fields
            .seps
            .into_iter()
            .map(|sep| sep.into_inner())
            .collect();
        let fields = Punctuated {
            items: fields.items,
            seps,
        };

        let rbrace_t = input.consume()?;

        Ok(Self {
            vis,
            name,
            fields,
            struct_t,
            lbrace_t,
            rbrace_t,
        })
    }
}

impl<I> Parse<I> for StructField
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            vis: input.parse()?,
            name: input.parse()?,
            colon_t: input.consume()?,
            ty: input.parse()?,
        })
    }
}

// impl<I> Parse<I> for StructFunction
// where
// I: Iterator<Item = Symbol>,
// {
// #[inline]
// fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
// // Parse visibility.
// let vis = input.parse()?;

// // Consume fn token.
// input.consume::<ttypes::Function>()?;

// // Parse function name.
// let name = input.parse()?;

// // Conusme left parenthesis.
// input.consume::<ttypes::LParen>()?;

// // Parse parameters.
// // Check if self parameter is present.
// let is_method = if input.peek_is(&Token::Ident("self".to_string())) {
// // Actually consume the self token.
// input.next();
// // Consume the comma.
// input.consume::<ttypes::Comma>()?;
// true
// } else {
// false
// };

// let params = input
// .parse::<Separated<FunctionParam, Rsv<ttypes::Comma>>>()?
// .items;

// // Conusme right parenthesis.
// input.consume::<ttypes::RParen>()?;

// // Parse the return type (if any).
// let return_type = match input.consume_opt::<ttypes::Arrow>()? {
// Some(_) => input.parse()?,
// None => Type::Primitive(PrimitiveType::Unit),
// };

// // Parse block.
// let body = input.parse()?;

// Ok(Self {
// vis,
// name,
// params,
// return_type,
// is_method,
// body,
// })
// }
// }
