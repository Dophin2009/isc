use std::collections::HashMap;
use std::hash::Hash;

use crate::error::ConstructionError;
use crate::ll::Parser as LLParser;

pub trait SymbolType = Clone + PartialEq + Eq;
pub trait Nonterminal = Hash + SymbolType;

pub type GrammarNoop<T, N> = Grammar<T, N, ()>;

#[derive(Debug, Clone)]
pub struct Grammar<T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    rules: HashMap<N, Vec<Rhs<T, N, A>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rhs<T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    pub body: Vec<Symbol<T, N>>,
    pub assoc: A,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    Terminal(T),
    Nonterminal(N),
}

impl<T, N> PartialEq<N> for Symbol<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    fn eq(&self, other: &N) -> bool {
        match self {
            Self::Terminal(_) => false,
            Self::Nonterminal(n) => n == other,
        }
    }
}

impl<T, N, A> Grammar<T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    // TODO: Return Result with custom error.
    pub fn new(rules: HashMap<N, Vec<Rhs<T, N, A>>>) -> Option<Self> {
        // Check that all nonterminals used in rule bodies have their own rules.
        if rules
            .iter()
            .flat_map(|(_, rhs)| rhs)
            .flat_map(|rhs| &rhs.body)
            .any(|sy| match sy {
                Symbol::Nonterminal(n) => rules.get(&n).is_none(),
                Symbol::Terminal(_) => false,
            })
        {
            None
        } else {
            Some(Self { rules })
        }
    }

    pub fn rules(&self) -> &HashMap<N, Vec<Rhs<T, N, A>>> {
        &self.rules
    }

    // pub fn predictive_parser(&self) -> Result<LLParser<T, N, A>, ConstructionError> {
    // LLParser::from_grammar(self)
    // }
}
