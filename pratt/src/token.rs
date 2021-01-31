#[derive(Debug, PartialEq)]
pub enum Token {
    Atom(String),

    Plus,
    Minus,
    Star,
    Slash,
    Exclamation,

    LParen,
    RParen,
    LBracket,
    RBracket,

    Error,
}
