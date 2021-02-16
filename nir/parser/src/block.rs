use crate::{ExpectedToken, Parse, ParseError, ParseInput, ParseResult, Symbol};
use ast::{
    Block, Break, Continue, ElseBranch, ExprStatement, ForLoop, IfBranch, IfElse, Statement,
    VarAssign, VarDeclaration, WhileLoop,
};
use lexer::{types as ttypes, Token};

impl<I> Parse<I> for Block
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        // Parse left brace.
        let lbrace_t = input.consume::<ttypes::LBrace>()?;

        // Parse statements.
        let mut statements = Vec::new();
        while !input.peek_is(&reserved!(RBrace)) {
            let statement = input.parse()?;
            statements.push(statement);
        }

        // Parse right brace.
        let rbrace_t = input.consume::<ttypes::RBrace>()?;

        Ok(Self {
            statements,
            lbrace_t,
            rbrace_t,
        })
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

        let peeked = match input.peek() {
            Some(peeked) => peeked,
            None => {
                input.error(ParseError::UnexpectedEof(expected()));
                return Err(());
            }
        };

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
            reserved!(If) => Self::IfElse(input.parse()?),
            // Can be variable assignment or expression.
            Token::Ident(_) => {
                let _ = input.peek_mult().unwrap();

                // Peek to see next symbol is equals for assignment.
                match input.peek_mult().map(|peeked| &peeked.0) {
                    Some(reserved!(Equ)) => {
                        input.reset_peek();
                        Self::VarAssign(input.parse()?)
                    }
                    _ => {
                        input.reset_peek();
                        let ret = Self::Expr(input.parse()?);
                        input.consume::<ttypes::Semicolon>()?;
                        ret
                    }
                }
            }
            // Otherwise, try to parse as expression.
            _ => {
                let expr = input.parse()?;
                input.consume::<ttypes::Semicolon>()?;

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
        Ok(Self {
            let_t: input.consume()?,
            lhs: input.parse()?,
            colon_t: input.consume()?,
            ty: input.parse()?,
            equ_t: input.consume()?,
            rhs: input.parse()?,
            semicolon_t: input.consume()?,
        })
    }
}

impl<I> Parse<I> for VarAssign
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            lhs: input.parse()?,
            equ_t: input.consume()?,
            rhs: input.parse()?,
            semicolon_t: input.consume()?,
        })
    }
}

impl<I> Parse<I> for ForLoop
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            for_t: input.consume()?,
            ident: input.parse()?,
            in_t: input.consume()?,
            range: input.parse()?,
            body: input.parse()?,
        })
    }
}

impl<I> Parse<I> for WhileLoop
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            while_t: input.consume()?,
            cond: input.parse()?,
            body: input.parse()?,
        })
    }
}

impl<I> Parse<I> for Break
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            break_t: input.consume()?,
            semicolon_t: input.consume()?,
        })
    }
}

impl<I> Parse<I> for Continue
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            continue_t: input.consume()?,
            semicolon_t: input.consume()?,
        })
    }
}

impl<I> Parse<I> for IfElse
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            head: input.parse()?,
        })
    }
}

impl<I> Parse<I> for IfBranch
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let if_t = input.consume()?;
        let cond = input.parse()?;
        let body = input.parse()?;

        let else_body = if input.peek_is(&reserved!(Else)) {
            Some(Box::new(input.parse()?))
        } else {
            None
        };

        Ok(Self {
            if_t,
            cond,
            body,
            else_body,
        })
    }
}

impl<I> Parse<I> for ElseBranch
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        let else_t = input.consume()?;

        let branch = if input.peek_is(&reserved!(If)) {
            Self::If {
                else_t,
                branch: input.parse()?,
            }
        } else {
            Self::Block {
                else_t,
                inner: input.parse()?,
            }
        };

        Ok(branch)
    }
}

impl<I> Parse<I> for ExprStatement
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> ParseResult<Self> {
        Ok(Self {
            expr: input.parse()?,
            semicolon_t: input.consume()?,
        })
    }
}
