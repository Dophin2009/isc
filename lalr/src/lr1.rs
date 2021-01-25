use crate::common::{LR1Action, LR1State, LR1Table, LRConflict};
use crate::grammar::{Grammar, Symbol};

use std::collections::BTreeMap;

impl<T, N, A> Grammar<T, N, A>
where
    T: Ord,
    N: Ord,
{
    /// Construct an SLR(1) parse table for the grammar.
    pub fn slr1_table<'a>(&'a self) -> Result<LR1Table<'a, T, N, A>, LRConflict<'a, T, N, A>> {
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
