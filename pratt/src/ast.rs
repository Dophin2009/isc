use std::fmt;

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
