use super::{Parse, ParseInput, Symbol};
use crate::ast::Visibility;
use crate::token::{Reserved, Token};

impl<I> Parse<I> for Visibility
where
    I: Iterator<Item = Symbol>,
{
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
