use crate::{Grammar, Rhs, Symbol};

use std::collections::{btree_set, BTreeSet, VecDeque};
use std::fmt::Debug;

impl<T, N, A> Grammar<T, N, A> {
    /// Compute the LR(0) item set.
    // fn lr0_set<'a>(&'a self) -> ItemSet<'a, T, N, A>
    // where
    // T: Eq + PartialEq,
    // N: Ord + PartialOrd,
    // Item<'a, T, N, A>: Ord + PartialOrd,
    // {
    // // Initialize item set to closure of {[S' -> S]}.
    // let mut set = ItemSet::new();
    // set.insert(Item {
    // lhs: &self.start,
    // rhs: &self.rules.get(&self.start).unwrap()[0],
    // pos: 0,
    // });
    // self.item_closure(&mut set);

    // let mut set_vec: VecDeque<_> = set.clone().into_iter().collect();

    // while let Some(item) = set_vec.pop_front() {
    // for sy in &item.rhs.body {
    // let mut goto = self.close_goto(&set, sy);
    // set.append(&mut goto);
    // }
    // }

    // set
    // }

    /// Compute the closure of items for the given item set.
    ///
    /// TODO: Find better, non-recursive way to write this?
    fn item_closure<'a>(&'a self, set: &mut ItemSet<'a, T, N, A>)
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
    fn close_goto<'a>(
        &'a self,
        set: &ItemSet<'a, T, N, A>,
        x: &'a Symbol<T, N>,
    ) -> ItemSet<'a, T, N, A>
    where
        T: PartialEq,
        N: Ord,
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
struct Item<'a, T: 'a, N: 'a, A: 'a> {
    pub lhs: &'a N,
    pub rhs: &'a Rhs<T, N, A>,

    /// Position of item, equal to index next symbol.
    pub pos: usize,
}

impl<'a, T: 'a, N: 'a, A: 'a> Item<'a, T, N, A> {
    /// Retrieves B for A -> a.Bb, or None if A -> a.
    fn next_symbol(&self) -> Option<&'a Symbol<T, N>> {
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

impl<'a, T: 'a, N: 'a, A: 'a> PartialEq for Item<'a, T, N, A>
where
    T: PartialEq,
    N: PartialEq,
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs && self.pos == other.pos
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Eq for Item<'a, T, N, A>
where
    T: Eq,
    N: Eq,
    A: Eq,
{
}

impl<'a, T: 'a, N: 'a, A: 'a> PartialOrd for Item<'a, T, N, A>
where
    T: PartialOrd,
    N: PartialOrd,
    A: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ((self.lhs, self.rhs, self.pos)).partial_cmp(&(other.lhs, other.rhs, other.pos))
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Ord for Item<'a, T, N, A>
where
    T: Ord,
    N: Ord,
    A: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ((self.lhs, self.rhs, self.pos)).cmp(&(other.lhs, other.rhs, other.pos))
    }
}

#[derive(Debug)]
struct ItemSet<'a, T: 'a, N: 'a, A: 'a> {
    pub items: BTreeSet<Item<'a, T, N, A>>,
}

impl<'a, T: 'a, N: 'a, A: 'a> ItemSet<'a, T, N, A>
where
    Item<'a, T, N, A>: Ord + PartialOrd,
{
    fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    fn insert(&mut self, item: Item<'a, T, N, A>) {
        self.items.insert(item);
    }

    fn append(&mut self, set: &mut Self) {
        self.items.append(&mut set.items);
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate through the items in this ItemSet.
    fn iter(&self) -> btree_set::Iter<Item<'a, T, N, A>> {
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

impl<'a, T: 'a, N: 'a, A: 'a> PartialEq for ItemSet<'a, T, N, A>
where
    T: PartialEq,
    N: PartialEq,
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Eq for ItemSet<'a, T, N, A>
where
    T: Eq,
    N: Eq,
    A: Eq,
{
}

impl<'a, T: 'a, N: 'a, A: 'a> PartialOrd for ItemSet<'a, T, N, A>
where
    T: PartialOrd,
    N: PartialOrd,
    A: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.items.partial_cmp(&other.items)
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> Ord for ItemSet<'a, T, N, A>
where
    T: Ord,
    N: Ord,
    A: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.items.cmp(&other.items)
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> IntoIterator for ItemSet<'a, T, N, A> {
    type Item = Item<'a, T, N, A>;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
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
}
