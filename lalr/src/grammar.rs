use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Grammar<T, N, A> {
    pub rules: BTreeMap<N, Vec<Rhs<T, N, A>>>,
    pub start: N,
}

pub type GrammarNoop<T, N> = Grammar<T, N, ()>;

#[derive(Debug, Clone)]
pub struct Rhs<T, N, A> {
    pub body: Vec<Symbol<T, N>>,
    pub assoc: A,
}

comparators!(Rhs(T, N, A), (T, N), (body));

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

pub type FirstSets<'a, T, N> = BTreeMap<&'a N, (BTreeSet<&'a T>, bool)>;
pub type FollowSets<'a, T, N> = BTreeMap<&'a N, BTreeSet<&'a T>>;

impl<T, N, A> Grammar<T, N, A>
where
    T: PartialEq,
    N: Ord,
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

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    /// Compute FOLLOW sets for the nonterminals in the grammar.
    pub fn follow_sets<'a>(
        &'a self,
        first_sets: Option<&'a FirstSets<'a, T, N>>,
    ) -> FollowSets<'a, T, N> {
        // Compute the FIRST sets if they're not given.
        let first_sets: Cow<'a, _> = match first_sets {
            Some(sets) => Cow::Borrowed(sets),
            None => {
                let sets = self.first_sets();
                Cow::Owned(sets)
            }
        };

        // Map of FOLLOW sets to return.
        let mut map: BTreeMap<_, _> = self
            .rules
            .iter()
            .map(|(lhs, _)| (lhs, BTreeSet::new()))
            .collect();

        // Loop until no FOLLOW sets have been modified.
        let mut changed = true;
        while changed {
            changed = false;
            for (lhs, rhs_set) in &self.rules {
                for rhs in rhs_set {
                    // Keep track of the following terminals; when an nonterminal is encountered,
                    // these terminals (which comes after it in the production body) will be added to
                    // that nonterminal's FOLLOW set.
                    let mut follow = BTreeSet::new();

                    // Flag to indicate whether or not all FIRST sets of previous (from the end)
                    // symbols have contained ε.
                    let mut open_end = true;

                    // Iterate through body symbols in reverse.
                    for sy in rhs.body.iter().rev() {
                        match *sy {
                            // When a terminal is encountered, add it to the tracked follow set.
                            // FIRST(X) = {X} when X is a terminal, so in the case of A -> αBβ,
                            // where β is a terminal, FIRST(β) = β is added to FOLLOW(B).
                            Symbol::Terminal(ref t) => {
                                open_end = false;
                                follow.clear();
                                follow.insert(t);
                            }
                            // When a nonterminal N is encountered,
                            Symbol::Nonterminal(ref n) => {
                                // Modify the FOLLOW set of this nonterminal.
                                let mut set = map.get(&n).unwrap().clone();

                                // Add the tracked terminals (that follow this nonterminal) to the
                                // FOLLOW set.
                                // For A -> αBβ, add FIRST(β) to FOLLOW(B).
                                for &t in follow.iter() {
                                    if set.insert(t) {
                                        changed = true;
                                    }
                                }

                                // For A -> αB or A -> αBβ where FIRST(β) contains ε, add FOLLOW(A)
                                // to FOLLOW(B).
                                if open_end {
                                    for &t in map.get(&lhs).unwrap() {
                                        if set.insert(t) {
                                            changed = true;
                                        }
                                    }
                                }

                                // Clear tracked terminals.
                                follow.clear();

                                // Add FIRST set of this nonterminal to FOLLOW set for the next
                                // (previous) nonterminal.
                                let n_first = first_sets.get(n).unwrap();
                                follow.extend(&n_first.0);

                                // If FIRST(n) does not contain ε,
                                if !n_first.1 {
                                    open_end = false;
                                }

                                map.insert(n, set);
                            }
                        }
                    }
                }
            }
        }

        map
    }

    /// Compute the FIRST sets for the nonterminals in the grammar.
    pub fn first_sets<'a>(&'a self) -> FirstSets<'a, T, N> {
        // Map of FIRST sets, with flag indicating whether or not the set contains ε.
        let mut map: BTreeMap<_, _> = self
            .rules
            .iter()
            .map(|(lhs, _)| (lhs, (BTreeSet::new(), false)))
            .collect();

        let mut changed = true;
        while changed {
            changed = false;
            for (lhs, rhs_set) in &self.rules {
                let mut first = map.remove(lhs).unwrap();
                for rhs in rhs_set {
                    if rhs.body.is_empty() {
                        first.1 = true
                    }

                    // For A -> X1 X2 X3 X4 ..., add FIRST(X1) to FIRST(A).
                    // Add FIRST(X2) if FIRST(X1) contains ε, and so on.
                    'inner: for sy in &rhs.body {
                        match *sy {
                            // FIRST(X) = {X} where X is a terminal.
                            Symbol::Terminal(ref t) => {
                                if first.0.insert(t) {
                                    changed = true;
                                }
                                break 'inner;
                            }
                            Symbol::Nonterminal(ref n) => {
                                let n_first = map.get(n).unwrap();
                                for t in &n_first.0 {
                                    if first.0.insert(t) {
                                        changed = true;
                                    }
                                }

                                if !n_first.1 {
                                    break 'inner;
                                }
                            }
                        }
                    }
                }
                map.insert(lhs, first);
            }
        }

        map
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Grammar, Rhs,
        Symbol::{Nonterminal as NT, Terminal as TT},
    };

    use Nonterminal::*;
    use Terminal::*;

    #[test]
    fn test_follow_sets() {
        let GrammarUtil { grammar, .. } = create_grammar();
        let follow_sets = grammar.follow_sets(None);

        let mut expected = BTreeMap::new();

        let right_paren: BTreeSet<_> = [RightParen].iter().collect();
        expected.insert(&D, right_paren.clone());
        expected.insert(&E, right_paren.clone());

        let plus_right_paren: BTreeSet<_> = [Plus, RightParen].iter().collect();
        expected.insert(&T, plus_right_paren.clone());
        expected.insert(&U, plus_right_paren.clone());

        expected.insert(&F, [Plus, Times, RightParen].iter().collect());

        assert_eq!(expected, follow_sets);
    }

    #[test]
    fn test_first_sets() {
        let GrammarUtil { grammar, .. } = create_grammar();
        let first_sets = grammar.first_sets();

        let mut expected = BTreeMap::new();

        let shared_set: BTreeSet<_> = [LeftParen, Id].iter().collect();
        expected.insert(&D, (shared_set.clone(), false));
        expected.insert(&T, (shared_set.clone(), false));
        expected.insert(&F, (shared_set, false));
        expected.insert(&E, ([Plus].iter().collect(), true));
        expected.insert(&U, ([Times].iter().collect(), true));

        assert_eq!(expected, first_sets);
    }

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    enum Nonterminal {
        D,
        E,
        T,
        U,
        F,
    }

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    enum Terminal {
        Plus,
        Times,
        LeftParen,
        RightParen,
        Id,
    }

    struct GrammarUtil {
        grammar: Grammar<Terminal, Nonterminal, ()>,
    }

    fn create_grammar() -> GrammarUtil {
        let mut rules = BTreeMap::new();

        // D -> T E
        let expr = Rhs::noop(vec![NT(T), NT(E)]);
        rules.insert(D, vec![expr]);

        // E -> + T E
        //    | ε
        let expr_prime = Rhs::noop(vec![TT(Plus), NT(T), NT(E)]);
        let expr_prime_empty = Rhs::noop(vec![]);
        rules.insert(E, vec![expr_prime, expr_prime_empty]);

        // T -> F U
        let term = Rhs::noop(vec![NT(F), NT(U)]);
        rules.insert(T, vec![term]);

        // U -> * F U
        //    | ε
        let term_prime = Rhs::noop(vec![TT(Times), NT(F), NT(U)]);
        let term_prime_empty = Rhs::noop(vec![]);
        rules.insert(U, vec![term_prime, term_prime_empty]);

        // F -> ( D )
        //    | id
        let factor = Rhs::noop(vec![TT(LeftParen), NT(D), TT(RightParen)]);
        let factor_id = Rhs::noop(vec![TT(Id)]);
        rules.insert(F, vec![factor, factor_id]);

        let grammar = Grammar::new(D, rules).unwrap();
        GrammarUtil { grammar }
    }
}
