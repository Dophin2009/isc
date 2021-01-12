use std::collections::BTreeMap;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Grammar<T, N, A> {
    pub rules: BTreeMap<N, Vec<Rhs<T, N, A>>>,
    pub start: N,
}

pub type GrammarNoop<T, N> = Grammar<T, N, ()>;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Rhs<T, N, A> {
    pub body: Vec<Symbol<T, N>>,
    pub assoc: A,
}

impl<T, N, A> Rhs<T, N, A> {
    pub fn new(body: Vec<Symbol<T, N>>, assoc: A) -> Self {
        Self { body, assoc }
    }
}

impl<T, N> Rhs<T, N, ()> {
    pub fn noop(body: Vec<Symbol<T, N>>) -> Self {
        Self { body, assoc: () }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Symbol<T, N> {
    Terminal(T),
    Nonterminal(N),
}

impl<T, N, A> Grammar<T, N, A>
where
    T: PartialEq,
    N: PartialEq + Ord + PartialOrd,
{
    // TODO: Return Result with custom error.
    pub fn new(start: N, rules: BTreeMap<N, Vec<Rhs<T, N, A>>>) -> Result<Self> {
        // Check that all nonterminals used in rule bodies have their own rules.
        // Vectors of Rhs may be empty to indicate A -> e.
        if !rules.iter().any(|(n, _)| *n == start) {
            Err(Error::NoStartRule)
        } else if rules
            .iter()
            .flat_map(|(_, rhs)| rhs)
            .flat_map(|rhs| &rhs.body)
            .any(|sy| match sy {
                Symbol::Nonterminal(n) => rules.get(&n).is_none(),
                Symbol::Terminal(_) => false,
            })
        {
            Err(Error::InvalidNonterminal)
        } else {
            Ok(Self { start, rules })
        }
    }
}
