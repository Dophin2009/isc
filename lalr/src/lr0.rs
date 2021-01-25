use crate::common::{LR1Action, LR1State, LR1Table, LRConflict};
use crate::{Grammar, Rhs, Symbol};

use std::collections::{btree_set, BTreeMap, BTreeSet, VecDeque};
use std::iter::FromIterator;

use itertools::Itertools;

/// An LR(0) state machine.
#[derive(Debug)]
pub struct LR0Automaton<'a, T: 'a, N: 'a, A: 'a> {
    /// The states of the machine and their transitions to other states.
    pub states: Vec<LR0State<'a, T, N, A>>,
    /// Index of the starting state.
    pub start: usize,
}

/// A state in the LR(0) automaton, containing a set of items.
#[derive(Debug)]
pub struct LR0State<'a, T: 'a, N: 'a, A: 'a> {
    /// Set of items represented by this state.
    pub items: ItemSet<'a, T, N, A>,
    pub transitions: BTreeMap<&'a Symbol<T, N>, usize>,
}

// LR0State comparators based off items only for performance.
comparators!(LR0State('a, T, N, A), (T, N), (items));

impl<'a, T: 'a, N: 'a, A: 'a> Clone for LR0State<'a, T, N, A> {
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
    pub fn lr0_automaton<'a>(&'a self) -> LR0Automaton<'a, T, N, A> {
        // Initialize item set to closure of {[S' -> S]}.
        let mut initial_set = ItemSet::new();
        initial_set.insert(Item {
            lhs: &self.start,
            rhs: &self.rules.get(&self.start).unwrap()[0],
            pos: 0,
        });
        self.item_closure(&mut initial_set);
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
                let goto_closure = self.close_goto(&state.items, &sy);
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
    pub fn item_closure<'a>(&'a self, set: &mut ItemSet<'a, T, N, A>)
    where
        N: Ord,
        Item<'a, T, N, A>: Ord,
    {
        let mut added = ItemSet::new();
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
                let new_item = Item {
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
            self.item_closure(&mut added);

            // Post-order insertion of new items.
            set.append(&mut added);
        }
    }

    /// Compute the GOTO(I, X) where I is a set of items and X is a grammar symbol, returning the
    /// set of all items [A -> aX.B] such that [A -> a.XB] is in I.
    pub fn close_goto<'a>(
        &'a self,
        set: &ItemSet<'a, T, N, A>,
        x: &'a Symbol<T, N>,
    ) -> ItemSet<'a, T, N, A>
    where
        T: PartialEq,
        Item<'a, T, N, A>: Ord,
    {
        // Collection of all new items.
        let mut closure = ItemSet::new();
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
            let mut new_set = ItemSet::new();
            new_set.insert(Item {
                lhs: item.lhs,
                rhs: item.rhs,
                pos: item.pos + 1,
            });
            self.item_closure(&mut new_set);

            // Add to total new item collection.
            closure.append(&mut new_set);
        }

        closure
    }
}

#[derive(Debug)]
pub struct Item<'a, T: 'a, N: 'a, A: 'a> {
    pub lhs: &'a N,
    pub rhs: &'a Rhs<T, N, A>,

    /// Position of item, equal to the index of the next symbol.
    pub pos: usize,
}

comparators!(Item('a, T, N, A), (T, N), (lhs, rhs, pos));

impl<'a, T: 'a, N: 'a, A: 'a> Item<'a, T, N, A> {
    /// Retrieves B for A -> a.Bb, or None if A -> a.
    pub fn next_symbol(&self) -> Option<&'a Symbol<T, N>> {
        self.rhs.body.get(self.pos)
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Clone for Item<'a, T, N, A> {
    fn clone(&self) -> Self {
        Self {
            lhs: self.lhs,
            rhs: self.rhs,
            pos: self.pos,
        }
    }
}

#[derive(Debug)]
pub struct ItemSet<'a, T: 'a, N: 'a, A: 'a> {
    pub items: BTreeSet<Item<'a, T, N, A>>,
}

comparators!(ItemSet('a, T, N, A), (T, N), (items));

impl<'a, T: 'a, N: 'a, A: 'a> ItemSet<'a, T, N, A>
where
    Item<'a, T, N, A>: Ord,
{
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, item: Item<'a, T, N, A>) -> bool {
        self.items.insert(item)
    }

    pub fn append(&mut self, set: &mut Self) {
        self.items.append(&mut set.items);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Iterate through the items in this ItemSet.
    pub fn iter(&self) -> btree_set::Iter<Item<'a, T, N, A>> {
        self.items.iter()
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Clone for ItemSet<'a, T, N, A> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> IntoIterator for ItemSet<'a, T, N, A> {
    type Item = Item<'a, T, N, A>;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> FromIterator<Item<'a, T, N, A>> for ItemSet<'a, T, N, A>
where
    T: Ord,
    N: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Item<'a, T, N, A>>,
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
    fn test_slr_table() {
        let GrammarUtil { grammar, .. } = create_grammar();
        let table = grammar.slr_table().unwrap();

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
    fn test_item_closure() {
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
        let mut set = ItemSet::new();

        set.insert(Item {
            lhs: &S,
            rhs: &start_rhs,
            pos: 0,
        });

        let mut expected = set.clone();
        expected.insert(Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &E,
            rhs: &t,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &T,
            rhs: &t_times_f,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &T,
            rhs: &f,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &F,
            rhs: &paren_e,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &F,
            rhs: &id,
            pos: 0,
        });

        grammar.item_closure(&mut set);

        assert_eq!(set, expected);
    }

    #[test]
    fn test_close_goto() {
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

        let mut set = ItemSet::new();
        set.insert(Item {
            lhs: &S,
            rhs: &start_rhs,
            pos: 1,
        });
        set.insert(Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 1,
        });

        let closure = grammar.close_goto(&set, &TT(Plus));

        let mut expected = ItemSet::new();
        expected.insert(Item {
            lhs: &E,
            rhs: &e_plus_t,
            pos: 2,
        });
        expected.insert(Item {
            lhs: &T,
            rhs: &t_times_f,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &T,
            rhs: &f,
            pos: 0,
        });
        expected.insert(Item {
            lhs: &F,
            rhs: &paren_e,
            pos: 0,
        });
        expected.insert(Item {
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
