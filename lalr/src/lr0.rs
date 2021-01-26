use crate::{Grammar, Rhs, Symbol};

use std::collections::{btree_set, BTreeMap, BTreeSet, VecDeque};
use std::iter::FromIterator;

use itertools::Itertools;

/// An LR(0) state machine.
#[derive(Debug)]
pub struct LR0Automaton<'g, T: 'g, N: 'g, A: 'g> {
    /// The states of the machine and their transitions to other states.
    pub states: Vec<LR0State<'g, T, N, A>>,
    /// Index of the starting state.
    pub start: usize,
}

/// A state in the LR(0) automaton, containing a set of items.
#[derive(Debug)]
pub struct LR0State<'g, T: 'g, N: 'g, A: 'g> {
    /// Set of items represented by this state.
    pub items: LR0ItemSet<'g, T, N, A>,
    pub transitions: BTreeMap<&'g Symbol<T, N>, usize>,
}

// LR0State comparators based off items only for performance.
comparators!(LR0State('g, T, N, A), (T, N), (items));

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR0State<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            transitions: self.transitions.clone(),
        }
    }
}

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    /// Compute the LR(0) item set.
    #[inline]
    pub fn lr0_automaton<'g>(&'g self) -> LR0Automaton<'g, T, N, A> {
        // Initialize item set to closure of {[S' -> S]}.
        let mut initial_set = LR0ItemSet::new();
        initial_set.insert(LR0Item {
            lhs: &self.start,
            rhs: &self.rules.get(&self.start).unwrap()[0],
            pos: 0,
        });
        self.lr0_closure(&mut initial_set);
        let initial_state = LR0State {
            items: initial_set.clone(),
            transitions: BTreeMap::new(),
        };

        // Vector of states and transitions of the final automaton.
        let mut states = Vec::new();
        // Maintain queue of items who rhs symbols to close on.
        let mut states_queue = VecDeque::new();
        // Set of existing sets of items and their indexes in states, to be used to check before
        // adding to vector of states.
        let mut existing_sets = BTreeMap::new();

        states.push(initial_state.clone());
        states_queue.push_back((initial_state, 0));
        existing_sets.insert(initial_set, 0);

        // For each set of items I in C
        while let Some((mut state, state_idx)) = states_queue.pop_front() {
            // For each grammar symbol X
            let symbols = state.items.iter().flat_map(|item| &item.rhs.body).dedup();
            for sy in symbols {
                // Compute GOTO(I, X).
                let goto_closure = self.lr0_goto(&state.items, &sy);
                if goto_closure.is_empty() {
                    continue;
                }

                // Check if GOTO(I, X) set already exists.
                match existing_sets.get(&goto_closure) {
                    Some(&dest_idx) => {
                        // If so, make transitions based off existing vector index.
                        state.transitions.insert(sy, dest_idx);
                    }
                    None => {
                        // Else, push new state to vec and queue.
                        let new_state = LR0State {
                            items: goto_closure.clone(),
                            transitions: BTreeMap::new(),
                        };
                        states.push(new_state.clone());
                        let new_idx = states.len() - 1;

                        // Create transition on current symbol to new state.
                        state.transitions.insert(sy, new_idx);

                        // Push state to queue to close on later.
                        states_queue.push_back((new_state, new_idx));
                        existing_sets.insert(goto_closure, new_idx);
                    }
                };
            }

            *states.get_mut(state_idx).unwrap() = state;
        }

        LR0Automaton { states, start: 0 }
    }

    /// Compute the closure of items for the given item set.
    ///
    /// TODO: Find better, non-recursive way to write this?
    #[inline]
    pub fn lr0_closure<'g>(&'g self, set: &mut LR0ItemSet<'g, T, N, A>)
    where
        N: Ord,
        LR0Item<'g, T, N, A>: Ord,
    {
        let mut added = LR0ItemSet::new();
        for item in set.iter() {
            // Add each item B -> .y for each item A -> a.Bb
            let next_symbol = match item.next_symbol() {
                Some(sy) => match sy {
                    Symbol::Nonterminal(n) => n,
                    Symbol::Terminal(_) => continue,
                },
                None => continue,
            };

            // Shouldn't panic as long as Grammar created with Grammar::new?
            for production in self.rules.get(next_symbol).unwrap() {
                let new_item = LR0Item {
                    lhs: next_symbol,
                    rhs: &production,
                    pos: 0,
                };
                if new_item != *item {
                    added.insert(new_item);
                }
            }
        }

        if !added.is_empty() {
            // Compute closure of items to be added to original set.
            self.lr0_closure(&mut added);

            // Post-order insertion of new items.
            set.append(&mut added);
        }
    }

    /// Compute the GOTO(I, X) where I is a set of items and X is a grammar symbol, returning the
    /// set of all items [A -> aX.B] such that [A -> a.XB] is in I.
    #[inline]
    pub fn lr0_goto<'g>(
        &'g self,
        set: &LR0ItemSet<'g, T, N, A>,
        x: &'g Symbol<T, N>,
    ) -> LR0ItemSet<'g, T, N, A>
    where
        T: PartialEq,
        LR0Item<'g, T, N, A>: Ord,
    {
        // Collection of all new items.
        let mut closure = LR0ItemSet::new();
        for item in set.iter() {
            // Get the symbol after the .
            let next_symbol = match item.next_symbol() {
                Some(sy) => sy,
                None => continue,
            };

            // Check that the next symbol is X.
            if *next_symbol != *x {
                continue;
            }

            // Compute closure for [A -> aX.B]
            let mut new_set = LR0ItemSet::new();
            new_set.insert(LR0Item {
                lhs: item.lhs,
                rhs: item.rhs,
                pos: item.pos + 1,
            });
            self.lr0_closure(&mut new_set);

            // Add to total new item collection.
            closure.append(&mut new_set);
        }

        closure
    }
}

#[derive(Debug)]
pub struct LR0Item<'g, T: 'g, N: 'g, A: 'g> {
    pub lhs: &'g N,
    pub rhs: &'g Rhs<T, N, A>,

    /// Position of item, equal to the index of the next symbol.
    pub pos: usize,
}

comparators!(LR0Item('g, T, N, A), (T, N), (lhs, rhs, pos));

impl<'g, T: 'g, N: 'g, A: 'g> LR0Item<'g, T, N, A> {
    /// Retrieves B for A -> a.Bb, or None if A -> a.
    #[inline]
    pub fn next_symbol(&self) -> Option<&'g Symbol<T, N>> {
        self.rhs.body.get(self.pos)
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR0Item<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            lhs: self.lhs,
            rhs: self.rhs,
            pos: self.pos,
        }
    }
}

#[derive(Debug)]
pub struct LR0ItemSet<'g, T: 'g, N: 'g, A: 'g> {
    pub items: BTreeSet<LR0Item<'g, T, N, A>>,
}

comparators!(LR0ItemSet('g, T, N, A), (T, N), (items));

impl<'g, T: 'g, N: 'g, A: 'g> LR0ItemSet<'g, T, N, A>
where
    LR0Item<'g, T, N, A>: Ord,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, item: LR0Item<'g, T, N, A>) -> bool {
        self.items.insert(item)
    }

    #[inline]
    pub fn append(&mut self, set: &mut Self) {
        self.items.append(&mut set.items);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Iterate through the items in this LR0ItemSet.
    #[inline]
    pub fn iter(&self) -> btree_set::Iter<LR0Item<'g, T, N, A>> {
        self.items.iter()
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR0ItemSet<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> IntoIterator for LR0ItemSet<'g, T, N, A> {
    type Item = LR0Item<'g, T, N, A>;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> FromIterator<LR0Item<'g, T, N, A>> for LR0ItemSet<'g, T, N, A>
where
    T: Ord,
    N: Ord,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = LR0Item<'g, T, N, A>>,
    {
        let mut items = BTreeSet::new();
        items.extend(iter);
        Self { items }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Grammar, Rhs,
        Symbol::{Nonterminal as NT, Terminal as TT},
    };

    use std::collections::BTreeMap;

    use Nonterminal::*;
    use Terminal::*;

    #[test]
    fn test_slr1_table() {
        let GrammarUtil { grammar, .. } = create_grammar();
        let table = grammar.slr1_table(&|_, _, _| 0).unwrap();

        assert_eq!(table.states.len(), 12);

        let initial_state = table.states.first().unwrap();
        assert!(initial_state.endmarker.is_none());

        assert_eq!(*initial_state.goto.get(&E).unwrap(), 1);
        assert_eq!(*initial_state.goto.get(&T).unwrap(), 2);
        assert_eq!(*initial_state.goto.get(&F).unwrap(), 3);
    }

    #[test]
    fn test_lr0_automaton() {
        let GrammarUtil { grammar, .. } = create_grammar();
        let automaton = grammar.lr0_automaton();

        assert_eq!(automaton.states.len(), 12);
    }

    #[test]
    fn test_lr0_closure() {
        let GrammarUtil {
            start_rhs,
            e_plus_t,
            t,
            t_times_f,
            f,
            paren_e,
            id,
            grammar,
        } = create_grammar();

        // Initial set of {[S -> .E]}
        let mut set = LR0ItemSet::new();

        set.insert(LR0Item {
            lhs: &S,
            rhs: &start_rhs,
            pos: 0,
        });

        let mut expected = set.clone();
        expected.insert(LR0Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &E,
            rhs: &t,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &T,
            rhs: &t_times_f,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &T,
            rhs: &f,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &F,
            rhs: &paren_e,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &F,
            rhs: &id,
            pos: 0,
        });

        grammar.lr0_closure(&mut set);

        assert_eq!(set, expected);
    }

    #[test]
    fn test_lr0_goto() {
        let GrammarUtil {
            start_rhs,
            e_plus_t,
            t: _,
            t_times_f,
            f,
            paren_e,
            id,
            grammar,
        } = create_grammar();

        let mut set = LR0ItemSet::new();
        set.insert(LR0Item {
            lhs: &S,
            rhs: &start_rhs,
            pos: 1,
        });
        set.insert(LR0Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 1,
        });

        let closure = grammar.lr0_goto(&set, &TT(Plus));

        let mut expected = LR0ItemSet::new();
        expected.insert(LR0Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 2,
        });
        expected.insert(LR0Item {
            lhs: &T,
            rhs: &t_times_f,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &T,
            rhs: &f,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &F,
            rhs: &paren_e,
            pos: 0,
        });
        expected.insert(LR0Item {
            lhs: &F,
            rhs: &id,
            pos: 0,
        });

        assert_eq!(closure, expected);
    }

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    enum Nonterminal {
        S,
        E,
        T,
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

    type GrammarRhs = Rhs<Terminal, Nonterminal, ()>;

    struct GrammarUtil {
        start_rhs: GrammarRhs,
        e_plus_t: GrammarRhs,
        t: GrammarRhs,
        t_times_f: GrammarRhs,
        f: GrammarRhs,
        paren_e: GrammarRhs,
        id: GrammarRhs,
        grammar: Grammar<Terminal, Nonterminal, ()>,
    }

    fn create_grammar() -> GrammarUtil {
        let mut rules = BTreeMap::new();

        // S -> E
        let start_rhs = Rhs::noop(vec![NT(E)]);
        rules.insert(S, vec![start_rhs.clone()]);

        // E -> E + T
        //    | T
        let e_plus_t = Rhs::noop(vec![NT(E), TT(Plus), NT(T)]);
        let t = Rhs::noop(vec![NT(T)]);
        rules.insert(E, vec![e_plus_t.clone(), t.clone()]);

        // T -> T * F
        //    | F
        let t_times_f = Rhs::noop(vec![NT(T), TT(Times), NT(F)]);
        let f = Rhs::noop(vec![NT(F)]);
        rules.insert(T, vec![t_times_f.clone(), f.clone()]);

        // F -> ( E )
        //    | id
        let paren_e = Rhs::noop(vec![TT(LeftParen), NT(E), TT(RightParen)]);
        let id = Rhs::noop(vec![TT(Id)]);
        rules.insert(F, vec![paren_e.clone(), id.clone()]);

        let grammar = Grammar::new(S, rules).unwrap();
        GrammarUtil {
            start_rhs,
            e_plus_t,
            t,
            t_times_f,
            f,
            paren_e,
            id,
            grammar,
        }
    }
}
