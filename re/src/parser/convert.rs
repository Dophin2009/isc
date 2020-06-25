use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CharType {
    Char(char),
    Newline,
    Whitespace,
    EndMarker,
}
