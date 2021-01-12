use crate::{Grammar, Rhs, Symbol};

use std::collections::{btree_set, BTreeSet};

impl<T, N, A> Grammar<T, N, A> {
    /// Compute the LR(0) item set.
    // fn lr0_set<'a>(&'a self) -> ItemSet<'a, T, N, A> {}

    /// Compute the closure of items for the given item set.
    ///
    /// TODO: Find better, non-recursive way to write this?
    fn item_closure<'a>(&'a self, set: &mut ItemSet<'a, T, N, A>)
    where
        N: Ord + PartialOrd,
        Item<'a, T, N, A>: Ord + PartialOrd,
    {
        // If passed-in set is empty, end recusion.
        if set.is_empty() {
            return;
        }

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
                added.insert(Item {
                    lhs: next_symbol,
                    rhs: &production,
                    pos: 0,
                });
            }
        }

        // Compute closure of items to be added to original set.
        self.item_closure(&mut added);

        // Post-order insertion of new items.
        set.append(&mut added);
    }

    /// Compute the GOTO(I, X) where I is a set of items and X is a grammar symbol, returning the
    /// set of all items [A -> aX.B] such that [A -> a.XB] is in I.
    fn close_goto<'a>(&'a self, set: &mut ItemSet<'a, T, N, A>, x: &'a N)
    where
        N: Ord + PartialOrd,
        Item<'a, T, N, A>: Ord + PartialOrd,
    {
        // If passed-in set is empty, end recursion.
        if set.is_empty() {
            return;
        }

        // Collection of all new items.
        let mut added = ItemSet::new();
        for item in set.iter() {
            // Get the symbol after the .
            let next_symbol = match item.next_symbol() {
                Some(sy) => match sy {
                    Symbol::Nonterminal(n) => n,
                    Symbol::Terminal(_) => continue,
                },
                None => continue,
            };

            // Check that the next symbol is X.
            if next_symbol != x {
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
            added.append(&mut new_set);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
}

impl<'a, T: 'a, N: 'a, A: 'a> ItemSet<'a, T, N, A>
where
    Item<'a, T, N, A>: Ord + PartialOrd,
{
    fn iter(&self) -> btree_set::Iter<Item<'a, T, N, A>> {
        self.items.iter()
    }
}

impl<'a, T: 'a, N: 'a, A: 'a> IntoIterator for ItemSet<'a, T, N, A>
where
    Item<'a, T, N, A>: Ord + PartialOrd,
{
    type Item = Item<'a, T, N, A>;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
