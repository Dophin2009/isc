use crate::error::ConstructionError;
use crate::grammar::{Grammar, Nonterminal, SymbolType};

#[derive(Debug, Clone)]
pub struct Parser<T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    grammar: Grammar<T, N, A>,
}

impl<T, N, A> Parser<T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    // pub fn from_grammar(grammar: &Grammar<T, N, A>) -> Result<Self, ConstructionError> {
    // let builder = ParserBuilder::new(grammar);
    // builder.build()
    // }
}

#[derive(Debug, Clone)]
pub struct ParserBuilder<'a, T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    grammar: &'a Grammar<T, N, A>,
}

impl<'a, T, N, A> ParserBuilder<'a, T, N, A>
where
    T: SymbolType,
    N: Nonterminal,
{
    fn new(grammar: &'a Grammar<T, N, A>) -> Self {
        Self { grammar }
    }

    // fn build(&self) -> Result<Parser<T, N, A>, ConstructionError> {
    // }

    fn first(&self) {}

    fn follow(&self) {}
}
