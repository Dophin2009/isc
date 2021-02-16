use crate::{ExpectedToken, Parse, ParseError, ParseInput, ParseResult, Symbol};

use ast::{
    keywords::{LParen, RParen},
    Expr, UnaryOp, UnaryOpExpr,
};
use lexer::Token;

impl<I> Parse<I> for Expr
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        expr_bp(input, 0)
    }
}

fn expr_bp<I>(input: &mut ParseInput<I>, bp: u8) -> ParseResult<Expr>
where
    I: Iterator<Item = Symbol>,
{
    let peeked = match input.peek() {
        Some(peeked) => peeked,
        None => {
            input.error(ParseError::UnexpectedEof(expected()));
            return Err(());
        }
    };

    let mut lhs = match &peeked.0 {
        // Identifier
        Token::Ident(_) => Expr::Var(input.parse()?),
        // Literal value
        Token::Literal(_) => expr_literal(input)?,
        // Unary operation
        reserved!(Minus) | reserved!(Exclamation) => Expr::UnaryOp(Box::new(expr_unary(input)?)),
        // Parenthesized expression
        reserved!(LParen) => expr_parenthesized(input)?,
        _ => {
            input.error(unexpectedeof!(
                ExpectedToken::Ident,
                ExpectedToken::LiteralOpaque,
                ereserved!(Minus),
                ereserved!(Exclamation),
                ereserved!(LParen)
            ));
            return Err(());
        }
    };

    loop {
        let peeked = match input.peek() {
            Some(peeked) => peeked,
            None => break,
        };
    }

    Ok(lhs)
}

fn expr_parenthesized<I>(input: &mut ParseInput<I>) -> ParseResult<Expr>
where
    I: Iterator<Item = Symbol>,
{
    input.consume::<LParen>()?;
    let inner = expr_bp(input, 0)?;
    input.consume::<RParen>()?;

    Ok(inner)
}

fn expr_literal<I>(input: &mut ParseInput<I>) -> ParseResult<Expr>
where
    I: Iterator<Item = Symbol>,
{
    let next = input.next_unwrap(|| vec![ExpectedToken::LiteralOpaque])?;
    let literal = match next.0 {
        Token::Literal(inner) => inner,
        _ => {
            input.error(unexpectedeof!(ExpectedToken::LiteralOpaque));
            return Err(());
        }
    };

    Ok(Expr::Literal(literal))
}

fn expr_unary<I>(input: &mut ParseInput<I>) -> ParseResult<UnaryOpExpr>
where
    I: Iterator<Item = Symbol>,
{
    let next = input.next_unwrap(|| vec![ereserved!(Minus), ereserved!(Exclamation)])?;
    let op = match next.0 {
        reserved!(Minus) => UnaryOp::Negative,
        reserved!(Exclamation) => UnaryOp::Not,
        _ => {
            input.error(unexpectedtoken!(
                next.1,
                next.0,
                ereserved!(Minus),
                ereserved!(Exclamation)
            ));
            return Err(());
        }
    };

    let ((), rbp) = prefix_binding_power(&op).unwrap();
    let operand = expr_bp(input, rbp)?;

    Ok(UnaryOpExpr { op, operand })
}

/// Return the binding powers (specifically the right) for prefix operators.
/// [`None`] is returned if `op` is not a valid prefix operator.
fn prefix_binding_power(op: &UnaryOp) -> Option<((), u8)> {
    let bp = match op {
        // Same binding power is probably fine, since they act on different types of operands?
        UnaryOp::Negative => ((), 9),
        UnaryOp::Not => ((), 9),
    };
    Some(bp)
}

fn expected() -> Vec<ExpectedToken> {
    vec![]
}
