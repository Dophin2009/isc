use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    Ast,
    Dfa,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Ast => write!(f, "malformed expression"),
            ParseError::Dfa => write!(f, "failed to construct DFA"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::Ast => None,
            ParseError::Dfa => None,
        }
    }
}
