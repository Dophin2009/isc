use lalr::{
    Grammar, RuleBody,
    Symbol::{Nonterminal, Terminal},
};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum N {
    S,
    A,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum T {
    A,
    B,
    C,
    D,
}

#[test]
fn test_left_recursion() {
    let mut rules = HashMap::new();
    rules.insert(
        N::S,
        vec![
            RuleBody(vec![Nonterminal(N::A), Terminal(T::A)]),
            RuleBody(vec![Terminal(T::B)]),
        ],
    );
    rules.insert(
        N::A,
        vec![
            RuleBody(vec![Nonterminal(N::A), Terminal(T::C)]),
            RuleBody(vec![Nonterminal(N::S), Terminal(T::D)]),
            RuleBody(vec![]),
        ],
    );

    let grammar = Grammar::new(rules).unwrap();
    // println!("{:#?}", grammar);

    let grammar = Grammar::eliminate_left_recursion(&grammar);
    println!("{:#?}", grammar);
}
