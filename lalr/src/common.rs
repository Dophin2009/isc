use crate::grammar::Rhs;

use std::collections::BTreeMap;

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
