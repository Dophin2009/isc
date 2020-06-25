use crate::parser::dfa::DFA;
use crate::parser::{self, error::ParseError};

#[derive(Debug)]
pub struct RegExp {
    dfa: DFA,
}

impl RegExp {
    pub fn matches(&self, s: &str) -> bool {
        self.dfa.matches(s)
    }

    pub fn new(expr: &str) -> Result<Self, ParseError> {
        let dfa = parser::regex_to_dfa(&expr)?;
        Ok(Self { dfa })
    }
}
