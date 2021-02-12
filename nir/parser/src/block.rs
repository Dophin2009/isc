use crate::{ExpectedToken, Parse, ParseError, ParseInput, ParseResult, Symbol};
use ast::{
    Block, Break, Continue, Expr, ForLoop, IfOnly, Statement, VarAssign, VarDeclaration, WhileLoop,
};
use lexer::{types as ttypes, Token};

use itertools::Itertools;

impl<I> Parse<I> for Block
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse left brace.
        input.consume::<ttypes::LBrace>()?;

        // Parse statements.
        let mut statements = Vec::new();
        while !input.peek_is(&reserved!(RBrace)) {
            let statement = input.parse()?;
            statements.push(statement);
        }

        // Parse right brace.
        input.consume::<ttypes::RBrace>()?;

        Ok(Self { statements })
    }
}

impl<I> Parse<I> for Statement
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        #[inline]
        fn expected() -> Vec<ExpectedToken> {
            vec![
                ereserved!(Let),
                ereserved!(For),
                ereserved!(While),
                ereserved!(Break),
                ereserved!(Continue),
                ereserved!(If),
                ExpectedToken::Ident,
                ExpectedToken::Expr,
            ]
        }

        let peeked = input.peek().ok_or_else(|| {
            input.error(ParseError::UnexpectedEof(expected()));
        })?;

        let statement = match peeked.0 {
            // Parse variable declaration.
            reserved!(Let) => Self::VarDeclaration(input.parse()?),
            // Parse for loop.
            reserved!(For) => Self::ForLoop(input.parse()?),
            // Parse while loop.
            reserved!(While) => Self::WhileLoop(input.parse()?),
            // Parse break statement.
            reserved!(Break) => Self::Break(input.parse()?),
            // Parse continue statement.
            reserved!(Continue) => Self::Continue(input.parse()?),
            // Parse if statement (without else).
            reserved!(If) => Self::IfOnly(input.parse()?),
            // Can be variable assignment or expression.
            Token::Ident(ident) => {
                // Peek to see next symbol is equals for assignment.
            }
            // Otherwise, try to parse as expression.
            _ => {
                let expr = input.parse()?;
                input.consume::<ttypes::Comma>()?;

                Self::Expr(expr)
            }
        };

        Ok(statement)
    }
}

impl<I> Parse<I> for VarDeclaration
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse let token.
        input.consume::<ttypes::Let>()?;

        let lhs = input.parse()?;

        // Parse colon.
        input.consume::<ttypes::Colon>()?;

        let ty = input.parse()?;

        // Parse equals token.
        input.consume::<ttypes::Equ>()?;

        let rhs = input.parse()?;

        // Parse semicolon.
        input.consume::<ttypes::Semicolon>()?;

        Ok(Self { lhs, ty, rhs })
    }
}

impl<I> Parse<I> for ForLoop
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse for token.
        input.consume::<ttypes::For>()?;

        let ident = input.parse()?;

        // Parse in token.
        input.consume::<ttypes::In>()?;

        let range = input.parse()?;
        let body = input.parse()?;

        Ok(Self { ident, range, body })
    }
}

impl<I> Parse<I> for WhileLoop
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse for token.
        input.consume::<ttypes::While>()?;

        let cond = input.parse()?;
        let body = input.parse()?;

        Ok(Self { cond, body })
    }
}

impl<I> Parse<I> for Break
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        input.consume::<ttypes::Break>()?;
        input.consume::<ttypes::Semicolon>()?;

        Ok(Self)
    }
}

impl<I> Parse<I> for Continue
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        input.consume::<ttypes::Continue>()?;
        input.consume::<ttypes::Semicolon>()?;

        Ok(Self)
    }
}

impl<I> Parse<I> for IfOnly
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse if token.
        input.consume::<ttypes::If>()?;

        let cond = input.parse()?;
        let body = input.parse()?;

        Ok(Self { cond, body })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AssignOrExpr {
    Assign(VarAssign),
    Expr(Expr),
}

fn parse_assign_or_expr<I>(input: &mut ParseInput<I>) -> ParseResult<AssignOrExpr>
where
    I: Iterator<Item = Symbol>,
{
    // Parse ident.
}
