use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Atom(String),

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

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Atom(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Exclamation => write!(f, "!"),
            Token::Question => write!(f, "?"),
            Token::Colon => write!(f, ":"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Error => write!(f, "<error>"),
        }
    }
}
