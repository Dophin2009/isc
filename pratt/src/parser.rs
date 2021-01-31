use crate::ast::{Atom, BinaryOp, Expr, UnaryOp};
use crate::token::Token;

use std::fmt;
use std::iter::Peekable;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse<I>(&self, input: I) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        let mut input = input.peekable();
        self.expr_bp(&mut input, 0)
    }

    fn expr_bp<I>(&self, input: &mut Peekable<I>, min_bp: u8) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        // Parse the lhs operand.
        let mut lhs = match self.peek(input)? {
            // Match an Atom token, then lhs is that Atom.
            Token::Atom(_) => self.expr_atom(input)?,
            // Match a left parenthesis, then parse the inside of the parentheses.
            Token::LParen => self.expr_parenthesized(input)?,
            // Match a prefix operator and parse the operand.
            Token::Minus => self.expr_negative(input)?,
            // If none of the above were matched, then return an error.
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.next(input)?,
                    vec![
                        ExpectedToken::Atom,
                        ExpectedToken::LParen,
                        ExpectedToken::Minus,
                    ],
                ))
            }
        };

        loop {
            // Peek the next token, and if EOF is reached, break from the loop.
            // Otherwise, continue parsing as infix or postfix operator.
            let op = match self.peek_optional(input)? {
                None => break,
                Some(token) => token,
            };

            lhs = match op {
                // Match the postfix ! operator; parse factorial expression.
                Token::Exclamation => {
                    let (lbp, ()) = self.postfix_binding_power(op).unwrap();
                    if lbp < min_bp {
                        break;
                    }

                    // Consume the exclamation point.
                    self.next(input)?;
                    Expr::UnaryOp(UnaryOp::Factorial, Box::new(lhs))
                }
                // Match the postfix array indexing operator; parse array indexing expression.
                Token::LBracket => {
                    let (lbp, ()) = self.postfix_binding_power(op).unwrap();
                    if lbp < min_bp {
                        break;
                    }

                    // Parse the [idx] expression.
                    let idx = self.array_index(input)?;
                    Expr::ArrayIndex(Box::new(lhs), Box::new(idx))
                }
                // Match an infix operator; parse the infix operation expression.
                Token::Plus | Token::Minus | Token::Star | Token::Slash => {
                    let (lbp, rbp) = self.infix_binding_power(op).unwrap();
                    if lbp < min_bp {
                        break;
                    }

                    let op = self.next(input)?;
                    let op = match op {
                        Token::Plus => BinaryOp::Add,
                        Token::Minus => BinaryOp::Subtract,
                        Token::Star => BinaryOp::Multiply,
                        Token::Slash => BinaryOp::Divide,
                        _ => std::unreachable!(),
                    };

                    let rhs = self.expr_bp(input, rbp)?;
                    Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs))
                }
                Token::Question => {
                    let (lbp, rbp) = self.infix_binding_power(op).unwrap();
                    if lbp < min_bp {
                        break;
                    }

                    // Consume the question mark.
                    self.next(input)?;

                    // Parse second operand.
                    let mhs = self.expr_bp(input, 0)?;

                    // Ensure that the next token is the ternary separator.
                    let next = self.next(input)?;
                    if next != Token::Colon {
                        return Err(ParseError::UnexpectedToken(
                            next,
                            vec![ExpectedToken::Colon],
                        ));
                    }

                    // Parse third operand.
                    let rhs = self.expr_bp(input, rbp)?;

                    Expr::Ternary(Box::new(lhs), Box::new(mhs), Box::new(rhs))
                }
                // If the peeked token is not an infix or postfix operator, break.
                _ => break,
            };
        }

        Ok(lhs)
    }

    /// Parse a parenthesized exprression.
    fn expr_parenthesized<I>(&self, input: &mut Peekable<I>) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        // Consume left parenthesis.
        let next = self.next(input)?;
        if Token::LParen != next {
            return Err(ParseError::UnexpectedToken(
                next,
                vec![ExpectedToken::LParen],
            ));
        }

        let inner = self.expr_bp(input, 0)?;

        // Check that the next is the closing right parenthesis.
        // If not, return an error.
        let next = self.next(input)?;
        if Token::RParen != next {
            return Err(ParseError::UnexpectedToken(
                next,
                vec![ExpectedToken::RParen],
            ));
        }

        Ok(inner)
    }

    /// Parse an expression for the negative prefix operator applied to another expression.
    fn expr_negative<I>(&self, input: &mut Peekable<I>) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        let next = self.next(input)?;
        if next != Token::Minus {
            return Err(ParseError::UnexpectedToken(
                next,
                vec![ExpectedToken::Minus],
            ));
        }

        let ((), rbp) = self.prefix_binding_power(&next).unwrap();
        let rhs = self.expr_bp(input, rbp)?;

        Ok(Expr::UnaryOp(UnaryOp::Negative, Box::new(rhs)))
    }

    /// Parse an expression for an atom.
    fn expr_atom<I>(&self, input: &mut Peekable<I>) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        Ok(Expr::Atom(self.atom(input)?))
    }

    /// Parse an array index expression, including the brackets, in an array indexing operation.
    fn array_index<I>(&self, input: &mut Peekable<I>) -> Result<Expr>
    where
        I: Iterator<Item = Token>,
    {
        let next = self.next(input)?;
        if next != Token::LBracket {
            return Err(ParseError::UnexpectedToken(
                next,
                vec![ExpectedToken::LBracket],
            ));
        }

        let idx = self.expr_bp(input, 0)?;

        let next = self.next(input)?;
        if next != Token::RBracket {
            return Err(ParseError::UnexpectedToken(
                next,
                vec![ExpectedToken::RBracket],
            ));
        }

        Ok(idx)
    }

    /// Parse an Atom.
    fn atom<I>(&self, input: &mut Peekable<I>) -> Result<Atom>
    where
        I: Iterator<Item = Token>,
    {
        let next = self.next(input)?;
        match next {
            Token::Atom(s) => Ok(Atom(s)),
            _ => Err(ParseError::UnexpectedToken(next, vec![ExpectedToken::Atom])),
        }
    }

    /// Return the binding powers (specifically the right) for prefix operators.
    /// [`None`] is returned if `op` is not a valid prefix operator.
    fn prefix_binding_power(&self, op: &Token) -> Option<((), u8)> {
        let bp = match op {
            Token::Minus => ((), 9),
            _ => return None,
        };
        Some(bp)
    }

    /// Return the binding powers (specifically the left) for postfix operators.
    /// [`None`] is returned if `op` is not a valid postfix operator.
    fn postfix_binding_power(&self, op: &Token) -> Option<(u8, ())> {
        let bp = match op {
            Token::Exclamation => (11, ()),
            Token::LBracket => (11, ()),
            _ => return None,
        };
        Some(bp)
    }

    /// Return the binding powers for infix operators.
    /// [`None`] is returned if `op` is not a valid infix operator.
    fn infix_binding_power(&self, op: &Token) -> Option<(u8, u8)> {
        let bp = match op {
            Token::Question => (4, 3),
            Token::Plus | Token::Minus => (5, 6),
            Token::Star | Token::Slash => (7, 8),
            _ => return None,
        };
        Some(bp)
    }

    /// Consume the next token from the input. Returns `ParseError::UnexpectedEof` if the end of
    /// the iterator was encountered, and `ParseError::LexerError` if the token was the error
    /// variant.
    fn next<I>(&self, input: &mut I) -> Result<Token>
    where
        I: Iterator<Item = Token>,
    {
        match input.next().ok_or(ParseError::UnexpectedEof)? {
            Token::Error => Err(ParseError::LexerError),
            t => Ok(t),
        }
    }

    /// Peek the next token in the input. Has the same error semantics as [`Self::next`]
    fn peek<'a, I>(&self, input: &'a mut Peekable<I>) -> Result<&'a Token>
    where
        I: Iterator<Item = Token>,
    {
        match input.peek().ok_or(ParseError::UnexpectedEof)? {
            Token::Error => Err(ParseError::LexerError),
            t => Ok(t),
        }
    }

    /// Peek the next token in the input. Has the same error semantics as [`Self::next`], except
    /// [`None`] is returned if the end of the input is encountered.
    fn peek_optional<'a, I>(&self, input: &'a mut Peekable<I>) -> Result<Option<&'a Token>>
    where
        I: Iterator<Item = Token>,
    {
        match input.peek() {
            Some(token) => match token {
                Token::Error => Err(ParseError::LexerError),
                _ => Ok(Some(token)),
            },
            None => Ok(None),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {:?}, expected one of {:?}", .0, .1)]
    UnexpectedToken(Token, Vec<ExpectedToken>),
    #[error("unexpected end-of-file")]
    UnexpectedEof,
    #[error("lexer error")]
    LexerError,
}

#[derive(Debug, Clone)]
pub enum ExpectedToken {
    Atom,

    Plus,
    Minus,
    Star,
    Slash,
    Exclamation,
    Question,
    Colon,

    LParen,
    RParen,
    LBracket,
    RBracket,
}

impl fmt::Display for ExpectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ExpectedToken::Atom => write!(f, "<atom>"),
            ExpectedToken::Plus => write!(f, "+"),
            ExpectedToken::Minus => write!(f, "-"),
            ExpectedToken::Star => write!(f, "*"),
            ExpectedToken::Slash => write!(f, "/"),
            ExpectedToken::Exclamation => write!(f, "!"),
            ExpectedToken::Question => write!(f, "?"),
            ExpectedToken::Colon => write!(f, ":"),
            ExpectedToken::LParen => write!(f, "("),
            ExpectedToken::RParen => write!(f, ")"),
            ExpectedToken::LBracket => write!(f, "["),
            ExpectedToken::RBracket => write!(f, "]"),
        }
    }
}
