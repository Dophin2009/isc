use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CharType {
    Char(char),
    Newline,
    Whitespace,
    EndMarker,
}

impl From<char> for CharType {
    fn from(c: char) -> Self {
        match c {
            ' ' => CharType::Whitespace,
            '\n' => CharType::Newline,
            _ => CharType::Char(c),
        }
    }
}
