use crate::{Parse, ParseInput, Symbol};

use ast::Visibility;
use lexer::{Reserved, Token};

impl<I> Parse<I> for Visibility
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        let vis = if input.peek_is(&Token::Reserved(Reserved::Pub)) {
            input.next();
            Visibility::Public
        } else {
            Visibility::Private
        };
        Ok(vis)
    }
}
