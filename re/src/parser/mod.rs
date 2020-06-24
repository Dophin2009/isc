pub mod error;

mod ast;

use error::ParseError;

/// This function attempts to implement **Algorithm 3.36**, the conversion of a regular expression
/// string directly to a DFA, from *Compilers: Principles, Techniques, and Tool*, Second Edition.
pub fn regex_to_dfa(expr: &str) -> Result<(), ParseError> {
    let _ast = ast::syntax_tree(expr)?;
    Ok(())
}
