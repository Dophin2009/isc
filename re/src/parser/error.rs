use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    Malformed,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Malformed => write!(f, "malformed expression"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::Malformed => None,
        }
    }
}
