pub mod ast;
pub mod automata;
pub mod error;

pub use error::ParseError;

use crate::dfa::DFA;

/// This function attempts to implement **Algorithm 3.36**, the conversion of a regular expression
/// string directly to a DFA, from *Compilers: Principles, Techniques, and Tool*, Second Edition.
pub fn regex_to_dfa(expr: &str) -> Result<DFA, ParseError> {
    println!("'{}'", expr);
    let ast = ast::syntax_tree(expr)?;
    println!("{:#?}", ast);
    let dfa = automata::tree_to_dfa(&ast)?;
    println!("");
    Ok(dfa)
}
