use super::Ident;

use std::collections::BTreeMap;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ScopeManager {
    pub stack: Vec<Scope>,
}

impl ScopeManager {
    #[inline]
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    #[inline]
    pub fn top(&self) -> Option<&Scope> {
        self.stack.last()
    }

    #[inline]
    pub fn top_mut(&mut self) -> Option<&mut Scope> {
        self.stack.last_mut()
    }

    #[inline]
    pub fn lookup(&self, ident: &String) -> Option<(&SymbolEntry, &Scope)> {
        self.stack
            .iter()
            .rev()
            .find_map(|st| st.get(ident).map(|entry| (entry, st)))
    }

    #[inline]
    pub fn push_new(&mut self) {
        self.stack.push(Scope::new())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Scope> {
        self.stack.pop()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct SymbolEntry {}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
pub struct Scope {
    pub inner: BTreeMap<String, SymbolEntry>,
}

impl Scope {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, ident: String, entry: SymbolEntry) -> Option<SymbolEntry> {
        self.inner.insert(ident, entry)
    }

    #[inline]
    pub fn insert_ident(&mut self, ident: Ident, entry: SymbolEntry) -> Option<SymbolEntry> {
        self.insert(ident.name_str().to_string(), entry)
    }

    /// Inserts into the symbol table and returns true if the given ident is not already present.
    #[inline]
    pub fn insert_nodup(&mut self, ident: String, entry: SymbolEntry) -> bool {
        if self.inner.contains_key(&ident) {
            false
        } else {
            self.inner.insert(ident, entry);
            true
        }
    }

    #[inline]
    pub fn insert_ident_nodup(&mut self, ident: Ident, entry: SymbolEntry) -> bool {
        self.insert_nodup(ident.name_str().to_string(), entry)
    }

    #[inline]
    pub fn get(&self, ident: &str) -> Option<&SymbolEntry> {
        self.inner.get(ident)
    }

    #[inline]
    pub fn get_mut(&mut self, ident: &str) -> Option<&mut SymbolEntry> {
        self.inner.get_mut(ident)
    }

    #[inline]
    pub fn remove(&mut self, ident: &str) -> Option<SymbolEntry> {
        self.inner.remove(ident)
    }

    #[inline]
    pub fn contains(&self, ident: &str) -> bool {
        self.inner.contains_key(ident)
    }
}

impl Default for Scope {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
