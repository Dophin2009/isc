use crate::{Parse, ParseInput, ParseResult, Rsv, Symbol};

use ast::{
    keywords::Comma, punctuated::Punctuated, scope::SymbolEntry, Ident, Struct, StructField,
};

impl<I> Parse<I> for Struct
where
    I: Iterator<Item = Symbol>,
{
    /// Parse a struct declaration.
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse visibility.
        let vis = input.parse()?;
        // Ensure next token is struct.
        let struct_t = input.consume()?;

        // Parse struct name.
        let name: Ident = input.parse()?;
        // Insert struct name into symbol table, emit error if already present.
        input.insert_ident_nodup(name.clone(), SymbolEntry {});

        // Ensure next token is opening brace.
        let lbrace_t = input.consume()?;

        // Parse fields.
        let fields = if input.peek_is(&reserved!(RBrace)) {
            Punctuated::default()
        } else {
            let fields = input.parse::<Punctuated<StructField, Rsv<Comma>>>()?;

            // TODO: ensure no duplicate fields.
            // if has_duplicates(&fields.items) {}

            let seps = fields
                .seps
                .into_iter()
                .map(|sep| sep.into_inner())
                .collect();
            Punctuated::new(fields.items, seps)
        };

        // Consume closing brace.
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

#[inline]
#[allow(dead_code)]
fn has_duplicates(fields: &[StructField]) -> bool {
    (1..fields.len()).any(|i| fields[i..].contains(&fields[i - 1]))
}
