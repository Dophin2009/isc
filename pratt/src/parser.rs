use crate::ast::Expr;
use crate::token::Token;

use std::fmt;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse<I>(&self, input: I) -> Result<Expr, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        let mut input = input.peekable();
        self.expr_bp(&mut input, 0)
    }

    #[must_use]
    fn expr_bp<I>(&self, input: &mut Peekable<I>, min_bp: u8) -> Result<Expr, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        let token = self.next(input)?;
        let lhs = match token {
            Token::Atom(s) => Expr::Atom(s),
            Token::LParen => {
                let lhs = self.expr_bp(input, 0)?;
                assert_eq!(self.next(input)?, Token::RParen);
                lhs
            }
            Token::Minus => {
                let ((), rbp) = self.prefix_binding_power(&token);
                self.expr_bp(input, rbp)?
            }
            _ => panic!(),
        };

        loop {
            let op = match input.peek() {
                None => break,
                Some(token) => match token {
                    Token::Atom(_) => return Err(ParseError::UnexpectedToken(token, vec![])),

                    _ => token,
                },
            };

            break;
        }

        Err(ParseError::UnexpectedEof)
    }

    /// Return the binding powers (specifically the right) for prefix operators.
    /// It's assumed that `op` is a valid prefix operator.
    fn prefix_binding_power(&self, op: &Token) -> ((), u8) {
        match op {
            Token::Minus => ((), 9),
            _ => panic!("bad operator: {:?}", op),
        }
    }

    fn next<I>(&self, input: &mut I) -> Result<Token, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        match input.next().ok_or(ParseError::UnexpectedEof)? {
            Token::Error => Err(ParseError::LexerError),
            t => Ok(t),
        }
    }

    fn peek<'a, I>(&self, input: &'a mut Peekable<I>) -> Result<&'a Token, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        match input.peek().ok_or(ParseError::UnexpectedEof)? {
            Token::Error => Err(ParseError::LexerError),
            t => Ok(t),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {:?}, expected one of {:?}", .0, .1)]
    UnexpectedToken(Token, Vec<ExpectedToken>),
    #[error("unexpected end-of-file")]
    UnexpectedEof,
    #[error("lexer error")]
    LexerError,
}

#[derive(Debug)]
pub enum ExpectedToken {
    Atom,

    Plus,
    Minus,
    Star,
    Slash,
    Exclamation,

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
            ExpectedToken::LParen => write!(f, "("),
            ExpectedToken::RParen => write!(f, ")"),
            ExpectedToken::LBracket => write!(f, "["),
            ExpectedToken::RBracket => write!(f, "]"),
        }
    }
}
