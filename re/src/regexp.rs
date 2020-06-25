use crate::parser::{self, error::ParseError};

#[derive(Debug)]
pub struct RegExp;

impl RegExp {
    pub fn matches(&self, _s: &str) -> bool {
        return false;
    }

    pub fn new(expr: &str) -> Result<Self, ParseError> {
        parser::regex_to_dfa(&expr)?;
        Ok(Self)
    }
}
