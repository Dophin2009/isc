use crate::lexer::Lexer;
use crate::parser::Parser;

use std::io::{self, BufRead};

use utf8_chars::BufReadCharsExt;

mod lexer {
    use llex::lexer;
    use ordered_float::NotNan;

    use Token::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Token {
        Let,

        Ident(String),
        Integer(i64),
        Float(NotNan<f64>),

        Equals,
        Plus,
        Minus,
        Star,
        Slash,

        LParen,
        RParen,
        Semicolon,

        Comment,

        Error,
    }

    lexer! {
        pub struct Lexer;
        pub fn stream;
        (text) -> Token, Token::Error;

        r"\s" => None,
        r"//[^\n]*" => Some(Comment),

        r"let" => Some(Let),

        r"[0-9]+" => {
            let i = text.parse().unwrap();
            Some(Integer(i))
        }
        r"[0-9]+\.([0-9])+" => {
            let f = text.parse().unwrap();
            Some(Float(NotNan::new(f).unwrap()))
        }
        r"[a-zA-Z_][a-zA-Z0-9_]*" => Some(Ident(text.to_string())),

        r"=" => Some(Equals),
        r"\+" => Some(Plus),
        r"-" => Some(Minus),
        r"\*" => Some(Star),
        r"/" => Some(Slash),

        r"\(" => Some(LParen),
        r"\)" => Some(RParen),
        r";" => Some(Semicolon),
    }
}

mod parser {
    use crate::lexer::Token;

    use lalrgen::parser;
    use ordered_float::NotNan;

    #[derive(Debug)]
    pub struct Program {
        stmts: Vec<Statement>,
    }

    #[derive(Debug)]
    pub enum Statement {
        Assign(String, Expr),
        Print(Expr),
    }

    #[derive(Debug)]
    pub enum Expr {
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
        Multiply(Box<Expr>, Box<Expr>),
        Divide(Box<Expr>, Box<Expr>),
        Negative(Box<Expr>),
        Var(String),
        Integer(i64),
        Float(NotNan<f64>),
    }

    parser! {
        pub struct Parser<Token>;

        Program: Program {
            Statements[stmts] => Ok(Program { stmts }),
        }

        Statements: Vec<Statement> {
            => Ok(vec![]),
            Statements[mut stmts] Statement[s] => {
                stmts.push(s);
                Ok(stmts)
            }
        }

        Statement: Statement {
             Let Ident(ident) Equals Expr[expr] Semicolon => Ok(Statement::Assign(ident, expr)),
        }

        Expr: Expr {
            Float(f) => Ok(Expr::Float(f)),
            Integer(i) => Ok(Expr::Integer(i)),
        }
    }
}

fn main() -> io::Result<()> {
    let lexer = Lexer::new();
    let parser = Parser::new();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    for line in stdin_lock.lines() {
        let line = line?;
        let chars = line.chars();

        let tokens = lexer.stream(chars).map(|item| item.token);

        let ast = parser.parse(tokens).unwrap();
        println!("{:#?}", ast);
    }

    Ok(())
}
