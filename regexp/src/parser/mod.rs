pub mod ast;
pub mod common;
pub mod dfa;
pub mod error;

pub use dfa::DFA;
pub use error::ParseError;

/// This function attempts to implement **Algorithm 3.36**, the conversion of a regular expression
/// string directly to a DFA, from *Compilers: Principles, Techniques, and Tool*, Second Edition.
pub fn regex_to_dfa(expr: &str) -> Result<DFA, ParseError> {
    let ast = ast::syntax_tree(expr)?;
    let dfa = DFA::from_ast(&ast)?;
    Ok(dfa)
}
