#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("grammar is malformed: {0}")]
    MalformedGrammar(#[from] MalformedGrammarError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MalformedGrammarError {
    #[error("starting nonterminal has no productions")]
    NoStartRule,
    #[error("nonterminal in right-hand side does not exist")]
    InvalidNonterminal,
}
