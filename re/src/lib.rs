mod parser;

use parser::ParseError;

#[derive(Debug)]
pub struct RegExp;

impl RegExp {
    pub fn matches(&self, _s: &str) -> bool {
        return false;
    }

    pub fn new(expr: &str) -> Result<Self, ParseError> {
        parser::regex_to_dfa(expr)?;
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use crate::RegExp;

    #[test]
    fn test_blank() {
        let exprs = ["", "()", "(())"];
        run_tests(&exprs);
    }

    #[test]
    fn test_single() {
        let exprs = [" ", "a", "b", "\""];
        run_tests(&exprs);
    }

    #[test]
    fn test_kleene() {
        let exprs = ["a*", "ab*", "(ab)", "(ab)*", "(ab*)*"];
        run_tests(&exprs);
    }

    #[test]
    fn test_alternate() {
        let exprs = ["a|b", "(a|b)", "a|b|c"];
        run_tests(&exprs);
    }
    #[test]
    fn test_concat() {
        let exprs = ["ab", "abc", "abb", "abcb"];
        run_tests(&exprs);
    }

    #[test]
    fn test_composite() {
        let exprs = ["(a|b)*abb", "ab(a|b)*abb"];
        run_tests(&exprs);
    }

    #[test]
    fn test_malformed() {
        let exprs = ["(", ")", "a(", "(()", "*", "|", "*a", "**", "a|", "a)*"];
        run_invalid_tests(&exprs);
    }

    fn run_tests(exprs: &[&str]) {
        exprs.iter().for_each(|&expr| {
            let _ = RegExp::new(expr).unwrap();
        });
    }

    fn run_invalid_tests(exprs: &[&str]) {
        exprs.iter().for_each(|&expr| {
            let _ = RegExp::new(expr).unwrap_err();
        });
    }
}
