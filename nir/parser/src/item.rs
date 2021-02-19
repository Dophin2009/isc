use crate::{Parse, ParseError, ParseInput, Symbol};

use ast::{scope::SymbolEntry, Function, Item, Struct};

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
                input.unexpected_token(next, vec![ereserved!(Struct), ereserved!(Function)]);
                return Err(());
            }
            Some(peeked) => peeked,
            None => {
                input.unexpected_eof(vec![ereserved!(Struct), ereserved!(Function)]);
                return Err(());
            }
        };

        let item = match &peeked.0 {
            // Parse a struct.
            reserved!(Struct) => {
                let mut s: Struct = input.parse()?;
                // Patch visibility.
                s.vis = vis;

                Item::Struct(s)
            }
            // Parse a function.
            reserved!(Function) => {
                let mut f: Function = input.parse()?;
                // Patch visibility.
                f.vis = vis;

                // Insert function name into symbol table, emit error if already present.
                let scope = input.sm.top_mut().unwrap();
                let fn_name = f.name.clone();
                if !scope.insert_ident_nodup(fn_name.clone(), SymbolEntry {}) {
                    input.error(ParseError::DuplicateIdent(fn_name));
                }

                Item::Function(f)
            }
            _ => {
                let next = input.next().unwrap();
                input.unexpected_token(
                    next,
                    vec![ereserved!(Pub), ereserved!(Struct), ereserved!(Function)],
                );
                return Err(());
            }
        };

        Ok(item)
    }
}
