use crate::grammar::FirstSets;
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

#[derive(Debug)]
pub struct LR1Parser<'a, T: 'a, N: 'a, A: 'a> {
    table: LR1Table<'a, T, N, A>,
}

#[derive(Debug)]
pub struct LR1Table<'a, T: 'a, N: 'a, A: 'a> {
    pub states: Vec<LR1State<'a, T, N, A>>,
    pub initial: usize,
}

/// State in an LR(1) automaton.
#[derive(Debug)]
pub struct LR1State<'a, T: 'a, N: 'a, A: 'a> {
    /// Map of actions to be taken on terminals. Terminals with no action have no map entry.
    pub actions: BTreeMap<&'a T, LR1Action<'a, T, N, A>>,
    /// Action to taken when lookahead is endmarker symbol.
    pub endmarker: Option<LR1Action<'a, T, N, A>>,
    /// Map of GOTO transitions to other states. Nonterminals with no GOTO have no map entry.
    pub goto: BTreeMap<&'a N, usize>,
}

#[derive(Debug)]
/// LR(1) action to be taken for some terminal.
pub enum LR1Action<'a, T: 'a, N: 'a, A: 'a> {
    /// Reduce a production.
    Reduce(&'a N, &'a Rhs<T, N, A>),
    /// Shift to some state.
    Shift(usize),
    /// Accept the input.
    Accept,
}

impl<'a, T: 'a, N: 'a, A: 'a> LR1State<'a, T, N, A> {
    /// Insert an action for a symbol, returning an [`LRConflict`] error some action already
    /// exists for that symbol.
    ///
    /// If `sy` is [`None`], it is interpreted as the endmarker terminal.
    pub fn set_action(
        &mut self,
        sy: Option<&'a T>,
        action: LR1Action<'a, T, N, A>,
    ) -> Result<(), LRConflict<'a, T, N, A>>
    where
        T: Ord,
    {
        match sy {
            Some(sy) => {
                // Check for existing action; if there is one, there is a conflict.
                // If no existing, set the action.
                match self.actions.get(sy) {
                    Some(existing) => {
                        // Only reduce-reduce and shift-reduce should occur.
                        let conflict =
                            Self::determine_conflict(existing, &action, Some(sy)).unwrap();
                        Err(conflict)
                    }
                    None => {
                        self.actions.insert(sy, action);
                        Ok(())
                    }
                }
            }
            // sy is endmarker terminal.
            None => match &self.endmarker {
                Some(existing) => {
                    let conflict = Self::determine_conflict(&existing, &action, None).unwrap();
                    Err(conflict)
                }
                None => {
                    self.endmarker = Some(action);
                    Ok(())
                }
            },
        }
    }

    fn determine_conflict(
        a1: &LR1Action<'a, T, N, A>,
        a2: &LR1Action<'a, T, N, A>,
        sy: Option<&'a T>,
    ) -> Option<LRConflict<'a, T, N, A>> {
        match *a1 {
            LR1Action::Reduce(n1, rhs1) => match *a2 {
                LR1Action::Reduce(n2, rhs2) => Some(LRConflict::ReduceReduce {
                    r1: (n1, rhs1),
                    r2: (n2, rhs2),
                }),
                LR1Action::Shift(dest2) => Some(LRConflict::ShiftReduce {
                    shift: (sy, dest2),
                    reduce: (n1, rhs1),
                }),
                _ => None,
            },
            LR1Action::Shift(dest1) => match *a2 {
                LR1Action::Reduce(n2, rhs2) => Some(LRConflict::ShiftReduce {
                    shift: (sy, dest1),
                    reduce: (n2, rhs2),
                }),
                _ => None,
            },
            _ => None,
        }
    }
}

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    /// Construct an LALR(1) parse table for the grammar.
    ///
    /// Implements **Algorithm 4.63** to efficiently compute the kernels of the LALR(1) collection
    /// of item sets for a grammar.
    pub fn lalr1_table<'a>(&'a self) -> Result<LR1Table<'a, T, N, A>, LRConflict<'a, T, N, A>> {
        // Compute the LR(0) item set.
        let mut lr0_automaton = self.lr0_automaton();

        // Compute the first sets.
        let first_sets = self.first_sets();

        // Remove non-kernel items.
        for state in lr0_automaton.states.iter_mut() {
            state.items = state
                .items
                .iter()
                // Kernel items include the initial item, S' -> .S, and all items whose dots are
                // not at the left end.
                .filter(|item| *item.lhs == self.start || item.pos != 0)
                .cloned()
                .collect();
        }

        for kernel in lr0_automaton.states {
            for item in kernel.items {}
        }

        Ok(LR1Table {
            states: Vec::new(),
            initial: 0,
        })
    }

    pub fn lr1_closure<'a>(
        &'a self,
        items: &mut BTreeSet<(Item<'a, T, N, A>, Option<&'a T>)>,
        first_sets: &FirstSets<'a, T, N>,
    ) {
        let mut added = BTreeSet::new();
        // For each item [A -> α.Bβ, a] in I
        for (item, lookahead) in items.iter() {
            // For each production B -> γ in G'
            let next_symbol = match item.next_symbol() {
                Some(sy) => match sy {
                    Symbol::Nonterminal(ref n) => n,
                    Symbol::Terminal(_) => continue,
                },
                None => continue,
            };
            for rhs in self.rules.get(next_symbol).unwrap() {
                // For each terminal t in FIRST(βa).
                // TODO: Memoize this.

                // Extract β from rhs.
                let beta = &rhs.body[(item.pos + 1)..];
                let mut first_set = (BTreeSet::new(), false);

                // Flag to determine when to stop computing FIRST.
                let mut beta_nullable = true;
                for sy in beta {
                    if !beta_nullable {
                        break;
                    }

                    match sy {
                        // For nonterminal n, add FIRST(n) to the total set.
                        Symbol::Nonterminal(ref n) => {
                            let (to_add, nullable) = first_sets.get(n).unwrap();
                            first_set.0.extend(to_add);

                            // No ε, so break from loop to stop adding to FIRST set.
                            // Also do not add the lookahead to FIRST.
                            if !nullable {
                                beta_nullable = false;
                            }
                        }
                        // For terminal t, add t to the FIRST set.
                        // Stop looping and do not add the lookahead to FIRST.
                        Symbol::Terminal(ref t) => {
                            first_set.0.insert(t);
                            beta_nullable = false;
                        }
                    }
                }

                // Only add lookahead a to first if β was nullable.
                if beta_nullable {
                    match lookahead {
                        // If lookahead is not $, add it.
                        Some(t) => {
                            first_set.0.insert(t);
                        }
                        // Otherwise, set endmarker flag to true.
                        None => {
                            first_set.1 = true;
                        }
                    }
                }

                for t in first_set.0 {
                    added.insert((
                        Item {
                            lhs: next_symbol,
                            rhs,
                            pos: 0,
                        },
                        Some(t),
                    ));
                }

                if first_set.1 {
                    added.insert((
                        Item {
                            lhs: next_symbol,
                            rhs,
                            pos: 0,
                        },
                        None,
                    ));
                }
            }
        }

        if !added.is_empty() {
            items.extend(added);
            self.lr1_closure(items, first_sets);
        }
    }

    /// Construct an SLR parse table for the grammar.
    pub fn slr_table<'a>(&'a self) -> Result<LR1Table<'a, T, N, A>, LRConflict<'a, T, N, A>> {
        let lr0_automaton = self.lr0_automaton();
        let follow_sets = self.follow_sets(None);

        // New states in the LR(1) table.
        let mut states = Vec::new();

        for lr0_state in lr0_automaton.states {
            let mut lr1_state = LR1State {
                actions: BTreeMap::new(),
                endmarker: None,
                goto: BTreeMap::new(),
            };

            for (sy, dest) in lr0_state.transitions {
                match *sy {
                    // If [A -> α.aβ] is in I_i and GOTO(I_i, a) = I_j and a is a terminal, then
                    // set ACTION[i, a] to "shift j".
                    Symbol::Terminal(ref t) => {
                        lr1_state.set_action(Some(t), LR1Action::Shift(dest))?;
                    }
                    // If GOTO(I_i, A) = I_j for nonterminal A, then GOTO[i, A] = j.
                    Symbol::Nonterminal(ref n) => {
                        lr1_state.goto.insert(n, dest);
                    }
                }
            }

            for item in lr0_state.items {
                // If [A -> α.] is in I_i, then set ACTION[i, a] to "reduce A -> α" for all a in
                // FOLLOW(A), unless A is S'.
                if item.pos == item.rhs.body.len() {
                    if *item.lhs != self.start {
                        let (follow_set, endmarker) = follow_sets.get(item.lhs).unwrap();
                        for sy in follow_set {
                            lr1_state
                                .set_action(Some(sy), LR1Action::Reduce(item.lhs, item.rhs))?;
                        }

                        if *endmarker {
                            lr1_state.set_action(None, LR1Action::Reduce(item.lhs, item.rhs))?;
                        }
                    } else {
                        // If [S' -> S.] is in I_i, then set ACTION[i, $] to "accept".
                        lr1_state.set_action(None, LR1Action::Accept)?;
                    }
                }
            }

            states.push(lr1_state);
        }

        Ok(LR1Table {
            states,
            // The initial state of the parser is the one constructed from the set of items
            // containing [S' -> .S].
            initial: lr0_automaton.start,
        })
    }

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

/// A conflict encountered when constructing an LR(1) parse table.
#[derive(Debug, Clone)]
pub enum LRConflict<'a, T: 'a, N: 'a, A: 'a> {
    /// Shift-reduce conflict
    ShiftReduce {
        /// Shift action involved in the conflict.
        /// 0: Terminal to shift on; endmarker terminal if [`None`].
        /// 1: Destination state of the shift.
        shift: (Option<&'a T>, usize),
        /// Reduce rule involved in the conflict.
        reduce: (&'a N, &'a Rhs<T, N, A>),
    },
    /// Reduce-reduce conflict
    ReduceReduce {
        r1: (&'a N, &'a Rhs<T, N, A>),
        r2: (&'a N, &'a Rhs<T, N, A>),
    },
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
