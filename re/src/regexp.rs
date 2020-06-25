use crate::parser::{self, dfa::DFA, error::*};

#[derive(Debug)]
pub struct RegExp {
    dfa: DFA,
}

impl RegExp {
    pub fn is_match(&self, s: &str) -> bool {
        self.dfa.is_match(s)
    }

    pub fn new(expr: &str) -> Result<Self, ParseError> {
        let dfa = parser::regex_to_dfa(&expr)?;
        Ok(Self { dfa })
    }
}
