use crate::{Parse, ParseError, ParseInput, ParseResult, Rsv, Symbol};

use ast::{
    keywords::Comma, punctuated::Punctuated, scope::SymbolEntry, Function, FunctionParam,
    PrimitiveType, PrimitiveTypeKind, Span, Type,
};

impl<I> Parse<I> for Function
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;
        // Parse fn token.
        let fn_t = input.consume()?;
        // Parse name.
        let name = input.parse()?;
        // Parse left parenthesis.
        let lparen_t = input.consume()?;

        // Push a new scope for the function.
        input.sm.push_new();

        // Parse function parameters.
        let params = if input.peek_is(&reserved!(RParen)) {
            // If next is right parenthesis, there are no parameters.
            Punctuated::default()
        } else {
            let params = input.parse::<Punctuated<FunctionParam, Rsv<Comma>>>()?;
            let seps = params
                .seps
                .into_iter()
                .map(|sep| sep.into_inner())
                .collect();

            // Insert parameters into symbol table.
            for param in &params.items {
                let param_name = &param.name;
                // If duplicate parameters, emit errors.
                let scope = input.sm.top_mut().unwrap();
                if !scope.insert_nodup(param_name.name_str().to_string(), SymbolEntry {}) {
                    input.error(ParseError::DuplicateIdent(param_name.clone()))
                }
            }

            Punctuated::new(params.items, seps)
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

        // Pop function scope.
        let scope = input.sm.pop().unwrap();

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

            scope,
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
