use crate::parser::Rsv;
use crate::{ExpectedToken, Parse, ParseError, ParseInput, ParseResult, Symbol};

use std::rc::Rc;

use ast::{
    keywords::{Comma, DoubleColon, LParen, RParen},
    punctuated::Punctuated,
    ArrayIndex, ArrayLiteral, BinOp, BinOpExpr, Expr, FunctionCall, Path, Spanned, UnaryOp,
    UnaryOpExpr,
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

impl<I> Parse<I> for FunctionCall
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse function path.
        let function = input.parse::<Punctuated<_, Rsv<DoubleColon>>>()?;
        let function = Punctuated::new(
            function.items,
            function.seps.into_iter().map(Rsv::into_inner).collect(),
        );

        // Parse left parenthesis.
        let lparen_t = input.consume()?;

        // Parse function arguments.
        let args = {
            let parsed_args = input.parse::<Punctuated<_, Rsv<Comma>>>()?;
            let items = parsed_args
                .items
                .into_iter()
                .map(|arg| input.expr_cache.get_or_insert(&arg))
                .collect();
            let seps = parsed_args.seps.into_iter().map(Rsv::into_inner).collect();
            Punctuated::new(items, seps)
        };

        // Parse right parenthesis.
        let rparen_t = input.consume()?;

        Ok(Self {
            function: Path { segments: function },
            args,
            lparen_t,
            rparen_t,
        })
    }
}

#[inline]
fn expr_bp<I>(input: &mut ParseInput<I>, min_bp: u8) -> ParseResult<Expr>
where
    I: Iterator<Item = Symbol>,
{
    // TODO: Actually fill in values
    #[inline]
    fn expected() -> Vec<ExpectedToken> {
        vec![]
    }

    // Peek next input.
    let peeked = match input.peek() {
        Some(peeked) => peeked,
        None => {
            input.error(ParseError::UnexpectedEof(expected()));
            return Err(());
        }
    };

    // Parse first left hand side.
    let mut lhs = match &peeked.0 {
        // Identifier or function call
        Token::Ident(_) => {
            let _ = input.peek_mult();

            match input.peek_mult() {
                Some(peeked) if peeked.0 == reserved!(DoubleColon) => {
                    input.reset_peek();
                    Expr::FunctionCall(input.parse()?)
                }
                // Nothing else; this is just an ident.
                _ => {
                    input.reset_peek();
                    let e = Expr::Var(Rc::new(input.parse()?));
                    input.expr_cache.get_or_insert(&e)
                }
            }
        }
        // Literal value
        Token::Literal(_) => expr_literal(input)?,
        // Array value
        reserved!(LBracket) => Expr::ArrayLiteral(Box::new(expr_array(input)?)),
        // Unary operation
        #[allow(unreachable_patterns)]
        reserved!(Minus) | reserved!(Exclamation) => Expr::UnaryOp(Box::new(expr_unary(input)?)),
        // Parenthesized expression
        reserved!(LParen) => expr_parenthesized(input)?,
        _ => {
            let next = input.next().unwrap();
            input.error(unexpectedtoken!(
                next.1,
                next.0,
                ExpectedToken::Ident,
                ExpectedToken::LiteralOpaque,
                ereserved!(Minus),
                ereserved!(Exclamation),
                ereserved!(LParen)
            ));
            return Err(());
        }
    };

    // Peek the next token, and if EOF is reached, break from the loop. Otherwise, continue
    // parsing as infix or postfix operator.
    while let Some(peeked) = input.peek() {
        lhs = match peeked.0 {
            // Handle array indexing operation.
            reserved!(LBracket) => {
                let (lbp, ()) = postfix_binding_power(&PostfixOp::ArrayIndex);
                if lbp < min_bp {
                    break;
                }

                let lbracket_t = input.consume()?;
                let index = input.parse()?;
                let rbracket_t = input.consume()?;

                // Parse the idx expression.
                Expr::ArrayIndex(Box::new(ArrayIndex {
                    array: lhs,
                    index,
                    lbracket_t,
                    rbracket_t,
                }))
            }
            // Handle infix operator
            // TODO: https://github.com/rust-lang/rust/issues/82012
            #[allow(unreachable_patterns)]
            reserved!(Plus)
            | reserved!(Minus)
            | reserved!(Star)
            | reserved!(Slash)
            | reserved!(Equ)
            | reserved!(Nequ)
            | reserved!(GtEqu)
            | reserved!(Gt)
            | reserved!(LtEqu)
            | reserved!(Lt)
            | reserved!(DoubleAmp)
            | reserved!(DoubleBar) => {
                #[inline]
                fn infix_expected() -> Vec<ExpectedToken> {
                    vec![
                        ereserved!(Plus),
                        ereserved!(Minus),
                        ereserved!(Star),
                        ereserved!(Slash),
                        ereserved!(Equ),
                        ereserved!(Nequ),
                        ereserved!(GtEqu),
                        ereserved!(Gt),
                        ereserved!(LtEqu),
                        ereserved!(Lt),
                        ereserved!(DoubleAmp),
                        ereserved!(DoubleBar),
                    ]
                }

                let next = input.next_unwrap(infix_expected)?;
                let op = match next.0 {
                    reserved!(Plus) => BinOp::Add,
                    reserved!(Minus) => BinOp::Subtract,
                    reserved!(Star) => BinOp::Multiply,
                    reserved!(Slash) => BinOp::Divide,
                    reserved!(Equ) => BinOp::Equ,
                    reserved!(Nequ) => BinOp::Nequ,
                    reserved!(GtEqu) => BinOp::GtEqu,
                    reserved!(Gt) => BinOp::Gt,
                    reserved!(LtEqu) => BinOp::LtEqu,
                    reserved!(Lt) => BinOp::Lt,
                    reserved!(DoubleAmp) => BinOp::And,
                    reserved!(DoubleBar) => BinOp::Or,
                    _ => {
                        input.error(ParseError::UnexpectedToken(next, infix_expected()));
                        return Err(());
                    }
                };

                let (lbp, rbp) = infix_binding_power(&op);
                if lbp < min_bp {
                    break;
                }

                let e2 = expr_bp(input, rbp)?;
                Expr::BinOp(Box::new(BinOpExpr {
                    op: Spanned::new(op, next.1),
                    e1: lhs,
                    e2,
                }))
            }
            _ => break,
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
            input.error(unexpectedtoken!(
                next.1,
                next.0,
                ExpectedToken::LiteralOpaque
            ));
            return Err(());
        }
    };

    Ok(Expr::Literal(Spanned::new(literal, next.1)))
}

fn expr_array<I>(input: &mut ParseInput<I>) -> ParseResult<ArrayLiteral>
where
    I: Iterator<Item = Symbol>,
{
    let lbracket_t = input.consume()?;

    let elements = input.parse::<Punctuated<_, Rsv<Comma>>>()?;
    let seps = elements.seps.into_iter().map(Rsv::into_inner).collect();
    let elements = Punctuated::new(elements.items, seps);

    let rbracket_t = input.consume()?;

    Ok(ArrayLiteral {
        lbracket_t,
        elements,
        rbracket_t,
    })
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

    let ((), rbp) = prefix_binding_power(&op);
    let operand = expr_bp(input, rbp)?;

    Ok(UnaryOpExpr {
        op: Spanned::new(op, next.1),
        operand,
    })
}

/// Return the binding powers (specifically the right) for prefix operators.
fn prefix_binding_power(op: &UnaryOp) -> ((), u8) {
    match op {
        // Same binding power is probably fine, since they act on different types of operands?
        UnaryOp::Negative => ((), 9),
        UnaryOp::Not => ((), 9),
    }
}

/// Return the binding powers for infix operators.
fn infix_binding_power(op: &BinOp) -> (u8, u8) {
    match op {
        BinOp::And | BinOp::Or => (3, 4),
        BinOp::Equ | BinOp::Nequ | BinOp::GtEqu | BinOp::Gt | BinOp::LtEqu | BinOp::Lt => (1, 2),
        BinOp::Add | BinOp::Subtract => (5, 6),
        BinOp::Multiply | BinOp::Divide => (7, 8),
    }
}

/// Return the binding powers (specifically the left) for postfix operators.
fn postfix_binding_power(op: &PostfixOp) -> (u8, ()) {
    match op {
        PostfixOp::ArrayIndex => (11, ()),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PostfixOp {
    ArrayIndex,
}
