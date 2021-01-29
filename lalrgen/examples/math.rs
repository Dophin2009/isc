use crate::lexer::Lexer;
use crate::parser::Parser;

use std::io::{self, Write};

mod lexer {
    use llex::lexer;
    use ordered_float::NotNan;

    use Token::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Token {
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
        UnOp(UnOp, Box<Expr>),
        BinOp(BinOp, Box<Expr>, Box<Expr>),
        Var(String),
        Integer(i64),
        Float(NotNan<f64>),
    }

    #[derive(Debug)]
    pub enum UnOp {
        Negative,
    }

    #[derive(Debug)]
    pub enum BinOp {
        Add,
        Sub,
        Multiply,
        Divide,
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
             Ident(ident) Equals Expr[expr] Semicolon => Ok(Statement::Assign(ident, expr)),
        }

        Expr: Expr {
            Expr[a] BinOp[op] Expr[b] => Ok(Expr::BinOp(op, Box::new(a), Box::new(b))),
            UnOp[op] Expr[a] => Ok(Expr::UnOp(op, Box::new(a))),
            Float(f) => Ok(Expr::Float(f)),
            Integer(i) => Ok(Expr::Integer(i)),
            Ident(ident) => Ok(Expr::Var(ident)),
        }

        UnOp: UnOp {
            Minus => Ok(UnOp::Negative),
        }

        BinOp: BinOp {
            Plus => Ok(BinOp::Add),
            Minus => Ok(BinOp::Sub),
            Star => Ok(BinOp::Multiply),
            Slash => Ok(BinOp::Divide),
        }
    }
}

fn main() -> io::Result<()> {
    let lexer = Lexer::new();
    let parser = Parser::new();

    let stdin = io::stdin();

    let mut buf = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;

        stdin.read_line(&mut buf)?;
        if buf.trim() == "quit" {
            break;
        }

        let chars = buf.chars();
        let tokens = lexer.stream(chars).map(|item| item.token);

        let ast = parser.parse(tokens).unwrap();
        println!("{:#?}", ast);
    }

    Ok(())
}
