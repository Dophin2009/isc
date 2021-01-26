use crate::grammar::{FirstSets, Grammar, Rhs, Symbol};

use std::iter::FromIterator;
use std::{
    cmp,
    collections::{btree_set, BTreeMap, BTreeSet, VecDeque},
};

use itertools::Itertools;

#[derive(Debug)]
pub struct LR1Parser<'g, T: 'g, N: 'g, A: 'g> {
    table: LR1Table<'g, T, N, A>,
}

#[derive(Debug)]
pub struct LR1Table<'g, T: 'g, N: 'g, A: 'g> {
    pub states: Vec<LR1State<'g, T, N, A>>,
    pub initial: usize,
}

/// State in an LR(1) automaton.
#[derive(Debug)]
pub struct LR1State<'g, T: 'g, N: 'g, A: 'g> {
    /// Map of actions to be taken on terminals. Terminals with no action have no map entry.
    pub actions: BTreeMap<&'g T, LR1Action<'g, T, N, A>>,
    /// Action to taken when lookahead is endmarker symbol.
    pub endmarker: Option<LR1Action<'g, T, N, A>>,
    /// Map of GOTO transitions to other states. Nonterminals with no GOTO have no map entry.
    pub goto: BTreeMap<&'g N, usize>,
}

#[derive(Debug)]
/// LR(1) action to be taken for some terminal.
pub enum LR1Action<'g, T: 'g, N: 'g, A: 'g> {
    /// Reduce a production.
    Reduce(&'g N, &'g Rhs<T, N, A>),
    /// Shift to some state.
    Shift(usize),
    /// Accept the input.
    Accept,
}

/// A conflict encountered when constructing an LR(1) parse table.
#[derive(Debug, Clone)]
pub enum LR1Conflict<'g, T: 'g, N: 'g, A: 'g> {
    /// Shift-reduce conflict
    ShiftReduce {
        /// Shift action involved in the conflict.
        /// 0: Terminal to shift on; endmarker terminal if [`None`].
        /// 1: Destination state of the shift.
        shift: (Option<&'g T>, usize),
        /// Reduce rule involved in the conflict.
        reduce: (&'g N, &'g Rhs<T, N, A>),
    },
    /// Reduce-reduce conflict
    ReduceReduce {
        r1: (&'g N, &'g Rhs<T, N, A>),
        r2: (&'g N, &'g Rhs<T, N, A>),
    },
}

#[derive(Debug)]
enum LR1ConflictResolution<'g, T: 'g, N: 'g, A: 'g> {
    Conflict(LR1Conflict<'g, T, N, A>),
    Override,
    Keep,
}

impl<'g, T: 'g, N: 'g, A: 'g> LR1State<'g, T, N, A> {
    /// Insert an action for a symbol, returning an [`LR1Conflict`] error some action already
    /// exists for that symbol.
    ///
    /// If `sy` is [`None`], it is interpreted as the endmarker terminal.
    #[inline]
    pub fn set_action<F>(
        &mut self,
        sy: Option<&'g T>,
        action: LR1Action<'g, T, N, A>,
        priority_of: &F,
    ) -> Result<(), LR1Conflict<'g, T, N, A>>
    where
        T: Ord,
        N: Ord,
        F: Fn(&N, &Rhs<T, N, A>, Option<&T>) -> i32,
    {
        match sy {
            Some(sy) => {
                // Check for existing action; if there is one, there is a conflict.
                // If no existing, set the action.
                match self.actions.get(sy) {
                    Some(existing) => {
                        // Only reduce-reduce and shift-reduce should occur.
                        match Self::determine_conflict(existing, &action, Some(sy), priority_of) {
                            LR1ConflictResolution::Conflict(conflict) => Err(conflict),
                            LR1ConflictResolution::Override => {
                                self.actions.insert(sy, action);
                                Ok(())
                            }
                            LR1ConflictResolution::Keep => Ok(()),
                        }
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
                    match Self::determine_conflict(&existing, &action, None, priority_of) {
                        LR1ConflictResolution::Conflict(conflict) => Err(conflict),
                        LR1ConflictResolution::Override => {
                            self.endmarker = Some(action);
                            Ok(())
                        }
                        LR1ConflictResolution::Keep => Ok(()),
                    }
                }
                None => {
                    self.endmarker = Some(action);
                    Ok(())
                }
            },
        }
    }

    #[inline]
    fn determine_conflict<F>(
        a1: &LR1Action<'g, T, N, A>,
        a2: &LR1Action<'g, T, N, A>,
        sy: Option<&'g T>,
        priority_of: &F,
    ) -> LR1ConflictResolution<'g, T, N, A>
    where
        T: Ord,
        N: Ord,
        F: Fn(&N, &Rhs<T, N, A>, Option<&T>) -> i32,
    {
        // TODO: check for same action; don't error on those
        match *a1 {
            LR1Action::Reduce(n1, rhs1) => match *a2 {
                LR1Action::Reduce(n2, rhs2) => {
                    if n1 == n2 && rhs1 == rhs2 {
                        LR1ConflictResolution::Keep
                    } else {
                        match priority_of(n1, rhs1, sy).cmp(&priority_of(n2, rhs2, sy)) {
                            // Existing has greater priority than new: keep
                            cmp::Ordering::Greater => LR1ConflictResolution::Keep,
                            // Existing has lower priority than new: override
                            cmp::Ordering::Less => LR1ConflictResolution::Override,
                            // Equal priority: conflict
                            cmp::Ordering::Equal => {
                                LR1ConflictResolution::Conflict(LR1Conflict::ReduceReduce {
                                    r1: (n1, rhs1),
                                    r2: (n2, rhs2),
                                })
                            }
                        }
                    }
                }
                LR1Action::Shift(dest2) => {
                    LR1ConflictResolution::Conflict(LR1Conflict::ShiftReduce {
                        shift: (sy, dest2),
                        reduce: (n1, rhs1),
                    })
                }
                _ => unreachable!(),
            },
            LR1Action::Shift(dest1) => match *a2 {
                LR1Action::Reduce(n2, rhs2) => {
                    LR1ConflictResolution::Conflict(LR1Conflict::ShiftReduce {
                        shift: (sy, dest1),
                        reduce: (n2, rhs2),
                    })
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct LR1Item<'g, T: 'g, N: 'g, A: 'g> {
    pub lhs: &'g N,
    pub rhs: &'g Rhs<T, N, A>,

    /// Position of item, equal to the index of the next symbol.
    pub pos: usize,
    pub lookahead: Option<&'g T>,
}

comparators!(LR1Item('g, T, N, A), (T, N), (lhs, rhs, pos, lookahead));

impl<'g, T: 'g, N: 'g, A: 'g> LR1Item<'g, T, N, A> {
    /// Retrieves B for A -> a.Bb, or None if A -> a.
    #[inline]
    pub fn next_symbol(&self) -> Option<&'g Symbol<T, N>> {
        self.rhs.body.get(self.pos)
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR1Item<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            lhs: self.lhs,
            rhs: self.rhs,
            pos: self.pos,
            lookahead: self.lookahead,
        }
    }
}

#[derive(Debug)]
pub struct LR1ItemSet<'g, T: 'g, N: 'g, A: 'g> {
    pub items: BTreeSet<LR1Item<'g, T, N, A>>,
}

comparators!(LR1ItemSet('g, T, N, A), (T, N), (items));

impl<'g, T: 'g, N: 'g, A: 'g> LR1ItemSet<'g, T, N, A>
where
    LR1Item<'g, T, N, A>: Ord,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, item: LR1Item<'g, T, N, A>) -> bool {
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

    /// Iterate through the items in this LR1ItemSet.
    #[inline]
    pub fn iter(&self) -> btree_set::Iter<LR1Item<'g, T, N, A>> {
        self.items.iter()
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR1ItemSet<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> IntoIterator for LR1ItemSet<'g, T, N, A> {
    type Item = LR1Item<'g, T, N, A>;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'g, T: 'g, N: 'g, A: 'g> FromIterator<LR1Item<'g, T, N, A>> for LR1ItemSet<'g, T, N, A>
where
    T: Ord,
    N: Ord,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = LR1Item<'g, T, N, A>>,
    {
        let mut items = BTreeSet::new();
        items.extend(iter);
        Self { items }
    }
}

#[derive(Debug)]
pub struct LR1Automaton<'g, T: 'g, N: 'g, A: 'g> {
    pub states: Vec<LR1AutomatonState<'g, T, N, A>>,
    pub start: usize,
}

#[derive(Debug)]
pub struct LR1AutomatonState<'g, T: 'g, N: 'g, A: 'g> {
    pub items: LR1ItemSet<'g, T, N, A>,
    pub transitions: BTreeMap<&'g Symbol<T, N>, usize>,
}

comparators!(LR1AutomatonState('g, T, N, A), (T, N), (items));

impl<'g, T: 'g, N: 'g, A: 'g> Clone for LR1AutomatonState<'g, T, N, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            transitions: self.transitions.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct LR1ItemCoreSet<'g, T: 'g, N: 'g, A: 'g> {
    items: BTreeSet<LR1ItemCore<'g, T, N, A>>,
}

comparators!(LR1ItemCoreSet('g, T, N, A), (T, N), (items));

impl<'g, T, N, A> From<LR1ItemSet<'g, T, N, A>> for LR1ItemCoreSet<'g, T, N, A>
where
    T: Ord,
    N: Ord,
{
    fn from(set: LR1ItemSet<'g, T, N, A>) -> Self {
        Self {
            items: set.items.into_iter().map_into().collect(),
        }
    }
}

#[derive(Debug, Clone)]
struct LR1ItemCore<'g, T: 'g, N: 'g, A: 'g> {
    lhs: &'g N,
    rhs: &'g Rhs<T, N, A>,
    pos: usize,
}

comparators!(LR1ItemCore('g, T, N, A), (T, N), (lhs, rhs, pos));

impl<'g, T, N, A> From<LR1Item<'g, T, N, A>> for LR1ItemCore<'g, T, N, A> {
    fn from(item: LR1Item<'g, T, N, A>) -> Self {
        Self {
            lhs: item.lhs,
            rhs: item.rhs,
            pos: item.pos,
        }
    }
}

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    #[inline]
    pub fn lalr1_table_by_lr1<'g, F>(
        &'g self,
        priority_of: &F,
    ) -> Result<LR1Table<'g, T, N, A>, LR1Conflict<'g, T, N, A>>
    where
        F: Fn(&N, &Rhs<T, N, A>, Option<&T>) -> i32,
    {
        // Construct C = the collection of sets of LR(1) items.
        let lr1_automaton = self.lr1_automaton();
        let lr1_states = &lr1_automaton.states;

        // For each core present among the set of LR(1) items, find all sets having that core, and
        // replace these sets by their union.

        // Vector of LR(1) states and the set of indexes to the original LR(1) item sets in
        // lr1_states.
        let mut states: Vec<(LR1State<'g, T, N, A>, BTreeSet<usize>)> = Vec::new();
        // Mapping of item core sets to indexes in the LR(1) state vector.
        let mut union_map: BTreeMap<LR1ItemCoreSet<'g, T, N, A>, usize> = BTreeMap::new();
        // Mapping of original LR(1) item set indexes (in lr1_states) to new LR(1) merged states
        // indexes (in states).
        let mut state_mapping = BTreeMap::new();

        // Group indexes of duplicates (by item cores).
        for (i, state) in lr1_states.iter().enumerate() {
            let items = &state.items;

            // Check for item core set in map.
            let item_cores: LR1ItemCoreSet<'g, T, N, A> = items.clone().into();
            match union_map.get(&item_cores) {
                // If already exists, then
                Some(state_idx) => {
                    let (_, indexes) = states.get_mut(*state_idx).unwrap();
                    indexes.insert(i);
                    state_mapping.insert(i, *state_idx);
                }
                None => {
                    let new_state = LR1State {
                        actions: BTreeMap::new(),
                        endmarker: None,
                        goto: BTreeMap::new(),
                    };
                    let mut indexes = BTreeSet::new();
                    indexes.insert(i);

                    states.push((new_state, indexes));

                    let state_idx = states.len() - 1;
                    union_map.insert(item_cores, state_idx);
                    state_mapping.insert(i, state_idx);
                }
            };
        }

        // item_cores : the LR(1) item core sets
        // state_idx  : index of corresponding new LR(1) state in `states`
        // state      : LR(1) state in `states` at index `state_idx`
        // indexes    : vector of indexes to original LR(1) item sets in `lr1_states`
        for (state, indexes) in states.iter_mut() {
            // Collect the associated automaton states.
            let associated_states: Vec<_> = indexes
                .iter()
                .map(|idx| lr1_states.get(*idx).unwrap())
                .collect();

            // Union the item sets for the current core/new LR(1) state.
            let item_union: BTreeSet<_> = associated_states
                .iter()
                .flat_map(|assoc| assoc.items.clone())
                .collect();

            let transitions: BTreeMap<_, _> = associated_states
                .iter()
                .flat_map(|assoc| &assoc.transitions)
                .collect();

            for (sy, dest) in transitions {
                let new_dest = state_mapping.get(dest).unwrap();
                match *sy {
                    Symbol::Terminal(ref t) => {
                        state.set_action(Some(t), LR1Action::Shift(*new_dest), priority_of)?;
                    }
                    Symbol::Nonterminal(ref n) => {
                        state.goto.insert(n, *new_dest);
                    }
                }
            }

            for item in item_union {
                if item.pos == item.rhs.body.len() {
                    if *item.lhs != self.start {
                        state.set_action(
                            item.lookahead,
                            LR1Action::Reduce(item.lhs, item.rhs),
                            priority_of,
                        )?;
                    } else if item.lookahead.is_none() {
                        state.set_action(None, LR1Action::Accept, priority_of)?;
                    }
                }
            }
        }

        Ok(LR1Table {
            states: states.into_iter().map(|x| x.0).collect(),
            // TODO: Not sure if this is actually correct
            initial: *state_mapping.get(&lr1_automaton.start).unwrap(),
        })
    }

    #[inline]
    pub fn lr1_table<'g, F>(
        &'g self,
        priority_of: &F,
    ) -> Result<LR1Table<'g, T, N, A>, LR1Conflict<'g, T, N, A>>
    where
        F: Fn(&N, &Rhs<T, N, A>, Option<&T>) -> i32,
    {
        let lr1_automaton = self.lr1_automaton();

        let mut states = Vec::new();

        for automaton_state in lr1_automaton.states {
            let mut lr1_state = LR1State {
                actions: BTreeMap::new(),
                endmarker: None,
                goto: BTreeMap::new(),
            };

            for (sy, dest) in automaton_state.transitions {
                match *sy {
                    // If [A -> α·aβ, b] is in I_i and GOTO(I_i, a) = I_j and a is a terminal, then
                    // set ACTION[i, a] to "shift j".
                    Symbol::Terminal(ref t) => {
                        lr1_state.set_action(Some(t), LR1Action::Shift(dest), priority_of)?;
                    }
                    // If GOTO(I_i, A) = I_j, then GOTO[i, A] = j.
                    Symbol::Nonterminal(ref n) => {
                        lr1_state.goto.insert(n, dest);
                    }
                }
            }

            for item in automaton_state.items {
                // If [A -> α·, a] is in I_i, A != S', then set ACTION[i, a] to "reduce A ->
                // α".
                if item.pos == item.rhs.body.len() {
                    if *item.lhs != self.start {
                        lr1_state.set_action(
                            item.lookahead,
                            LR1Action::Reduce(item.lhs, item.rhs),
                            priority_of,
                        )?;
                    } else if item.lookahead.is_none() {
                        // If [S' -> S·, $] is in I_i, then set ACTION[i, $] to "accept".
                        lr1_state.set_action(None, LR1Action::Accept, priority_of)?;
                    }
                }
            }

            states.push(lr1_state);
        }

        Ok(LR1Table {
            states,
            initial: lr1_automaton.start,
        })
    }

    #[inline]
    pub fn lr1_automaton<'g>(&'g self) -> LR1Automaton<'g, T, N, A> {
        // Initialize item set to closure of {[S' -> S]}.
        let mut initial_set = LR1ItemSet::new();
        initial_set.insert(LR1Item {
            lhs: &self.start,
            rhs: &self.rules.get(&self.start).unwrap()[0],
            pos: 0,
            lookahead: None,
        });

        let first_sets = self.first_sets();
        self.lr1_closure(&mut initial_set, &first_sets);

        let initial_state = LR1AutomatonState {
            items: initial_set.clone(),
            transitions: BTreeMap::new(),
        };

        let mut states = Vec::new();
        let mut states_queue = VecDeque::new();
        let mut existing_sets = BTreeMap::new();

        states.push(initial_state.clone());
        states_queue.push_back((initial_state, 0));
        existing_sets.insert(initial_set, 0);

        while let Some((mut state, state_idx)) = states_queue.pop_front() {
            let symbols = state.items.iter().flat_map(|item| &item.rhs.body).dedup();
            for sy in symbols {
                // Compute GOTO(I, X)
                let goto_closure = self.lr1_goto(&state.items, &sy, &first_sets);
                if goto_closure.is_empty() {
                    continue;
                }

                // Check if GOTO(I, X) set already exists.
                match existing_sets.get(&goto_closure) {
                    Some(&dest_idx) => {
                        state.transitions.insert(sy, dest_idx);
                    }
                    None => {
                        let new_state = LR1AutomatonState {
                            items: goto_closure.clone(),
                            transitions: BTreeMap::new(),
                        };
                        states.push(new_state.clone());
                        let new_idx = states.len() - 1;

                        state.transitions.insert(sy, new_idx);
                        states_queue.push_back((new_state, new_idx));
                        existing_sets.insert(goto_closure, new_idx);
                    }
                };
            }

            *states.get_mut(state_idx).unwrap() = state;
        }

        LR1Automaton { states, start: 0 }
    }

    #[inline]
    pub fn lr1_goto<'g>(
        &'g self,
        set: &LR1ItemSet<'g, T, N, A>,
        x: &'g Symbol<T, N>,
        first_sets: &FirstSets<'g, T, N>,
    ) -> LR1ItemSet<'g, T, N, A> {
        let mut new_set = LR1ItemSet::new();

        // For each item [A -> α·Xβ, a] in I, add item [A -> aX·β, a] to set J.
        for item in set.iter() {
            let post_dot = match item.next_symbol() {
                Some(sy) => sy,
                None => continue,
            };

            if *post_dot != *x {
                continue;
            }

            new_set.insert(LR1Item {
                pos: item.pos + 1,
                ..*item
            });
        }

        self.lr1_closure(&mut new_set, first_sets);
        return new_set;
    }

    /// Compute the LR(1) closure set for the given LR(1) item set.
    #[inline]
    pub fn lr1_closure<'g>(
        &'g self,
        set: &mut LR1ItemSet<'g, T, N, A>,
        first_sets: &FirstSets<'g, T, N>,
    ) {
        let mut changed = true;
        while changed {
            changed = false;
            let mut added = BTreeSet::new();
            // For each item [A -> α·Bβ, a] in I where B is a nonterminal.
            for item in set.iter() {
                // Extract B.
                let b = match item.next_symbol() {
                    Some(sy) => match sy {
                        Symbol::Nonterminal(ref n) => n,
                        Symbol::Terminal(_) => continue,
                    },
                    None => continue,
                };

                // For each production B -> γ in G'.
                let b_productions = self.rules.get(b).unwrap();

                if !b_productions.is_empty() {
                    // Compute FIRST(βa).
                    let first_beta_a = {
                        // Extract β (all symbols after B).
                        let a = item.lookahead;
                        let beta = &item.rhs.body[(item.pos + 1)..];

                        let mut first_set = BTreeSet::new();
                        let mut nullable = true;
                        // Add to FIRST set until the current symbol's FIRST set is not nullable.
                        for sy in beta {
                            if !nullable {
                                break;
                            }

                            match sy {
                                // FIRST(t) where t is a terminal is never nullable, add to set and
                                // stop.
                                Symbol::Terminal(ref t) => {
                                    first_set.insert(t);
                                    nullable = false;
                                }
                                Symbol::Nonterminal(ref n) => {
                                    // Get FIRST(n) of the nonterminal n and add its terminals to
                                    // the total FIRST set.
                                    let (sy_first, sy_nullable) = first_sets.get(n).unwrap();
                                    first_set.extend(sy_first);
                                    if !sy_nullable {
                                        nullable = false;
                                    }
                                }
                            }
                        }

                        let mut first_set: BTreeSet<_> =
                            first_set.into_iter().map(|t| Some(t)).collect();

                        // If all of β was nullable, consider the terminal a.
                        if nullable {
                            first_set.insert(a);
                        }

                        first_set
                    };

                    for rhs in b_productions {
                        // For each terminal b in FIRST(βa), add [B -> ·γ, b] to set I.
                        for bt in &first_beta_a {
                            added.insert(LR1Item {
                                lhs: b,
                                rhs,
                                pos: 0,
                                lookahead: bt.clone(),
                            });
                        }
                    }
                }
            }

            if !added.is_empty() {
                for item in added.into_iter() {
                    if set.insert(item) {
                        changed = true;
                    }
                }
            }
        }
    }

    /// Construct an SLR(1) parse table for the grammar.
    #[inline]
    pub fn slr1_table<'g, F>(
        &'g self,
        priority_of: &F,
    ) -> Result<LR1Table<'g, T, N, A>, LR1Conflict<'g, T, N, A>>
    where
        F: Fn(&N, &Rhs<T, N, A>, Option<&T>) -> i32,
    {
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
                        lr1_state.set_action(Some(t), LR1Action::Shift(dest), priority_of)?;
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
                            lr1_state.set_action(
                                Some(sy),
                                LR1Action::Reduce(item.lhs, item.rhs),
                                priority_of,
                            )?;
                        }

                        if *endmarker {
                            lr1_state.set_action(
                                None,
                                LR1Action::Reduce(item.lhs, item.rhs),
                                priority_of,
                            )?;
                        }
                    } else {
                        // If [S' -> S.] is in I_i, then set ACTION[i, $] to "accept".
                        lr1_state.set_action(None, LR1Action::Accept, priority_of)?;
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
}

#[cfg(test)]
mod test_grammar_4_55 {
    use crate::{
        Grammar, Rhs,
        Symbol::{Nonterminal as NT, Terminal as TT},
    };

    use std::collections::BTreeMap;

    use Nonterminal::*;
    use Terminal::*;

    #[test]
    fn test_lalr1_table_by_lr1() {
        let grammar = create_grammar();
        let table = grammar.lalr1_table_by_lr1(&|_, _, _| 0).unwrap();

        assert_eq!(7, table.states.len());
    }

    #[test]
    fn test_lr1_table() {
        let grammar = create_grammar();

        let table = grammar.lr1_table(&|_, _, _| 0).unwrap();

        assert_eq!(10, table.states.len());
    }

    fn create_grammar() -> Grammar<Terminal, Nonterminal, ()> {
        let mut rules = BTreeMap::new();

        // E -> S
        let start_rhs = Rhs::noop(vec![NT(S)]);
        rules.insert(E, vec![start_rhs]);

        // S -> C C
        let cc = Rhs::noop(vec![NT(C), NT(C)]);
        rules.insert(S, vec![cc]);

        // C -> x C
        //    | y
        let x_c = Rhs::noop(vec![TT(X), NT(C)]);
        let y = Rhs::noop(vec![TT(Y)]);
        rules.insert(C, vec![x_c, y]);

        return Grammar::new(E, rules).unwrap();
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Nonterminal {
        E,
        S,
        C,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Terminal {
        X,
        Y,
    }
}

#[cfg(test)]
mod test_grammar_4_49 {
    use super::*;
    use crate::{
        Grammar, Rhs,
        Symbol::{Nonterminal as NT, Terminal as TT},
    };

    use std::collections::BTreeMap;

    use Nonterminal::*;
    use Terminal::*;

    #[test]
    fn test_lr1_closure() {
        let mut rules = BTreeMap::new();

        // E -> S
        let start_rhs = Rhs::noop(vec![NT(S)]);
        rules.insert(E, vec![start_rhs.clone()]);

        // S -> L = R
        //    | R
        let l_eq_r = Rhs::noop(vec![NT(L), TT(Equ), NT(R)]);
        let r = Rhs::noop(vec![NT(R)]);
        rules.insert(S, vec![l_eq_r, r]);

        // L -> * R
        //    | id
        let deref_r = Rhs::noop(vec![TT(Deref), NT(R)]);
        let id = Rhs::noop(vec![TT(Id)]);
        rules.insert(L, vec![deref_r, id]);

        // R -> L
        let l = Rhs::noop(vec![NT(L)]);
        rules.insert(R, vec![l]);

        let grammar = Grammar::new(E, rules).unwrap();

        // Compute CLOSURE({[E -> ·S, #]})
        let mut initial_set = LR1ItemSet::new();
        initial_set.insert(LR1Item {
            lhs: &E,
            rhs: &start_rhs,
            pos: 0,
            lookahead: None,
        });

        grammar.lr1_closure(&mut initial_set, &grammar.first_sets());

        assert_eq!(8, initial_set.items.len());
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Nonterminal {
        E,
        S,
        L,
        R,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Terminal {
        Equ,
        Deref,
        Id,
    }
}
