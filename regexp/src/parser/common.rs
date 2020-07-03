use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CharType {
    Char(char),
    Newline,
    Whitespace,
    EndMarker,
    Any,
}

impl CharType {
    pub fn from_plain(c: char) -> Self {
        match c {
            ' ' => CharType::Whitespace,
            '\n' => CharType::Newline,
            _ => CharType::Char(c),
        }
    }
}
