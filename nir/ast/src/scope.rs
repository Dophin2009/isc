use super::Ident;

use std::collections::BTreeMap;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

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
    pub fn insert<F: AsSymbolKey>(&mut self, ident: F, entry: SymbolEntry) -> Option<SymbolEntry> {
        self.inner.insert(ident.as_string(), entry)
    }

    #[inline]
    pub fn insert_nodup<F: AsSymbolKey>(&mut self, ident: F, entry: SymbolEntry) -> bool {
        let s = ident.as_string();
        match self.inner.get_mut(&s) {
            Some(_) => false,
            None => {
                self.inner.insert(s, entry);
                true
            }
        }
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

pub trait AsSymbolKey {
    fn as_string(self) -> String;
}

impl AsSymbolKey for &str {
    fn as_string(self) -> String {
        self.to_string()
    }
}

impl AsSymbolKey for String {
    fn as_string(self) -> String {
        self
    }
}

impl AsSymbolKey for Ident {
    fn as_string(self) -> String {
        self.name_str().to_string()
    }
}
