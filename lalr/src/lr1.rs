use crate::grammar::{Grammar, Rhs, Symbol};

use std::collections::BTreeMap;

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    /// Construct an SLR(1) parse table for the grammar.
    pub fn slr1_table<'g>(&'g self) -> Result<LR1Table<'g, T, N, A>, LRConflict<'g, T, N, A>> {
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
}

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
pub enum LRConflict<'g, T: 'g, N: 'g, A: 'g> {
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

impl<'g, T: 'g, N: 'g, A: 'g> LR1State<'g, T, N, A> {
    /// Insert an action for a symbol, returning an [`LRConflict`] error some action already
    /// exists for that symbol.
    ///
    /// If `sy` is [`None`], it is interpreted as the endmarker terminal.
    pub fn set_action(
        &mut self,
        sy: Option<&'g T>,
        action: LR1Action<'g, T, N, A>,
    ) -> Result<(), LRConflict<'g, T, N, A>>
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
        a1: &LR1Action<'g, T, N, A>,
        a2: &LR1Action<'g, T, N, A>,
        sy: Option<&'g T>,
    ) -> Option<LRConflict<'g, T, N, A>> {
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
