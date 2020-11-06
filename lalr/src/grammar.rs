use crate::left_recursion::GrammarEliminateLR;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;
use std::vec;

pub trait SymbolType = Clone + PartialEq + Eq;
pub trait Nonterminal = Hash + SymbolType;

#[derive(Debug, Clone)]
pub struct Grammar<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    rules: HashMap<N, Vec<RuleBody<T, N>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleBody<T, N>(pub Vec<Symbol<T, N>>)
where
    T: SymbolType,
    N: Nonterminal;

impl<T, N> RuleBody<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    pub fn first(&self) -> Option<&Symbol<T, N>> {
        self.0.first()
    }
}

impl<T, N> IntoIterator for RuleBody<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    type Item = Symbol<T, N>;
    type IntoIter = vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, N> FromIterator<Symbol<T, N>> for RuleBody<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    fn from_iter<I: IntoIterator<Item = Symbol<T, N>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
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

impl<T, N> Grammar<T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    // TODO: Return Result with custom error.
    pub fn new(rules: HashMap<N, Vec<RuleBody<T, N>>>) -> Option<Self> {
        Some(Self { rules })
    }

    pub fn eliminate_left_recursion(grammar: &Self) -> Self {
        GrammarEliminateLR::new_grammar(grammar)
    }

    pub fn rules(&self) -> &HashMap<N, Vec<RuleBody<T, N>>> {
        &self.rules
    }
}
