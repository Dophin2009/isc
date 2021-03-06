#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("starting nonterminal has no productions")]
    NoStartRule,
    #[error("nonterminal in right-hand side does not exist")]
    InvalidNonterminal,
}

pub type Result<T> = std::result::Result<T, Error>;
