use crate::lexer::Lexer;
use crate::parser::{ParseError, Parser};

use std::io::{self, Write};

fn main() -> Result<(), Error> {
    let lexer = Lexer::new();
    let parser = Parser::new();

    let stdin = io::stdin();

    let mut buf = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;

        stdin.read_line(&mut buf)?;
        let tokens = lexer.stream(buf.chars()).map(|item| item.token);

        let ast = parser.parse(tokens)?;
        println!("{}", ast);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] io::Error),
    #[error("error during parsing")]
    ParseError(#[from] ParseError),
}

mod lexer {
    llex::lexer! {
        pub struct Lexer;
        pub fn stream;
        (text) -> Token, Token::Error;

        r"\s"   => None,

        r"[0-9]+(\.[0-9]+)?" => {
            let f = text.parse().unwrap();
            Some(Token::Number(f))
        },

        r"\+"   => Some(Token::Plus),
        r"-"    => Some(Token::Minus),
        r"\*"   => Some(Token::Star),
        r"/"    => Some(Token::Slash),

        r"\("   => Some(Token::LParen),
        r"\)"   => Some(Token::RParen),

        r"\["   => Some(Token::LBracket),
        r"\]"   => Some(Token::RBracket),
    }

    #[derive(Debug)]
    pub enum Token {
        Ident(String),
        Number(f64),

        Plus,
        Minus,
        Star,
        Slash,

        LParen,
        RParen,
        LBracket,
        RBracket,

        Error,
    }
}

mod parser {
    use crate::lexer::Token;

    use std::fmt;

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
            Err(ParseError::UnexpectedEof)
        }
    }

    #[derive(Debug)]
    pub enum Expr {
        UnaryOp(UnaryOp, Box<Expr>),
        BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
        ArrayIndex(Atom, Box<Expr>),
        Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
        Atom(String),
    }

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Expr::UnaryOp(op, e1) => write!(f, "({} {})", op, e1),
                Expr::BinaryOp(op, e1, e2) => write!(f, "({} {} {})", op, e1, e2),
                Expr::ArrayIndex(ident, idx) => write!(f, "([ {} {})", ident, idx),
                Expr::Ternary(pred, e1, e2) => write!(f, "(? {} {} {})", pred, e1, e2),
                Expr::Atom(s) => write!(f, "{}", s),
            }
        }
    }

    #[derive(Debug)]
    pub struct Atom(pub String);

    impl fmt::Display for Atom {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Debug)]
    pub enum UnaryOp {
        Negative,
        Factorial,
    }

    impl fmt::Display for UnaryOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                UnaryOp::Negative => write!(f, "-"),
                UnaryOp::Factorial => write!(f, "!"),
            }
        }
    }

    #[derive(Debug)]
    pub enum BinaryOp {
        Add,
        Subtract,
        Multiply,
        Divide,
    }

    impl fmt::Display for BinaryOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                BinaryOp::Add => write!(f, "+"),
                BinaryOp::Subtract => write!(f, "-"),
                BinaryOp::Multiply => write!(f, "*"),
                BinaryOp::Divide => write!(f, "/"),
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ParseError {
        #[error("unexpected token encountered")]
        UnexpectedToken(Token, Vec<Token>),
        #[error("unexpected end-of-file")]
        UnexpectedEof,
    }
}
