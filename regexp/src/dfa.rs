use crate::convert::CharType;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug)]
pub struct DFA {
    pub start: u32,
    pub trans: DTran,
    pub accepting: HashSet<u32>,
}

impl DFA {
    // Determines if the given string is accepted by this DFA by stepping through the DFA
    // character-by-character.
    pub fn is_match(&self, s: &str) -> bool {
        let mut pos = self.start;
        for c in s.chars() {
            let char_type = CharType::from_plain(c);
            pos = match self.trans.get(&pos, &char_type) {
                Some(next) => *next,
                None => match self.trans.get(&pos, &CharType::Any) {
                    Some(next) => *next,
                    None => return false,
                },
            };
        }

        return self.accepting.contains(&pos);
    }
}

pub type DTran = Table<u32, CharType, u32>;

#[derive(Debug)]
pub struct Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    map: HashMap<T, HashMap<U, V>>,
}

impl<T, U, V> Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, row: T, col: U, val: V) -> Option<V> {
        match self.map.get_mut(&row) {
            Some(c) => c.insert(col, val),
            None => {
                let mut map = HashMap::new();
                map.insert(col, val);
                self.map.insert(row, map);
                None
            }
        }
    }

    pub fn get(&self, row: &T, col: &U) -> Option<&V> {
        match self.map.get(row) {
            Some(c) => c.get(col),
            None => None,
        }
    }
}
