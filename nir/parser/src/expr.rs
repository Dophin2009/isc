use crate::{ExpectedToken, Parse, ParseError, ParseInput, ParseResult, Symbol};
use ast::Expr;
use lexer::{types as ttypes, Token};

impl<I> Parse<I> for Expr
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        expr_bp(input)
    }
}

fn expr_bp<I>(input: &mut ParseInput<I>) -> ParseResult<Expr>
where
    I: Iterator<Item = Symbol>,
{
    let peeked = input.peek().ok_or_else(|| {
        input.error(ParseError::UnexpectedEof(expected()));
    })?;

    let mut lhs = match peeked.0 {
        Token::Ident(_) => Expr::Var(input.parse()?),
        Token::Literal(literal) => Expr::
    };
}

fn expected() -> Vec<ExpectedToken> {
    vec![]
}
