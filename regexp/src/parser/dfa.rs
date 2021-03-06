use super::ast::{Operator, SyntaxTree};
use super::common::CharType;
use super::error::ParseError;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug)]
pub struct DFA {
    pub start: u32,
    pub trans: DTran,
    pub accepting: HashSet<u32>,
}

impl DFA {
    pub fn from_ast(ast: &SyntaxTree) -> Result<Self, ParseError> {
        tree_to_dfa(ast)
    }

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

        self.accepting.contains(&pos)
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

#[derive(Debug)]
struct DState {
    label: u32,
    positions: HashSet<u32>,
}

impl PartialEq<HashSet<u32>> for DState {
    fn eq(&self, other: &HashSet<u32>) -> bool {
        self.positions == *other
    }
}

#[derive(Debug)]
struct DFABase {
    root_firstpos: HashSet<u32>,
    leaves: LeafLookup,
}

type LeafLookup = HashMap<u32, AugmentedNode>;

macro_rules! hash_set {
    () => {
        HashSet::new()
    };
    ( $( $x:expr ),* ) => {{
        let mut set = HashSet::new();
        $(set.insert($x);)*
        set
    }};
}

/// Implements steps 2 and 3 of **Algorithm 3.36** in *Compilers: Principles,
/// Techniques, and Tool*, Second Edition.
fn tree_to_dfa(tree: &SyntaxTree) -> Result<DFA, ParseError> {
    let base = match calculate_functions(tree)? {
        Some(b) => b,
        None => {
            return Ok(DFA {
                start: 0,
                trans: Table::new(),
                accepting: hash_set![0],
            });
        }
    };

    let mut label = 0;

    // Initially only one state present, unmarked: firstpos of the root node.
    let mut marked_states = Vec::new();
    let mut unmarked_states = VecDeque::new();
    unmarked_states.push_back(DState {
        label,
        positions: base.root_firstpos.clone(),
    });

    let mut s_op = unmarked_states.pop_front();
    let mut dfa = DFA {
        start: 0,
        trans: Table::new(),
        accepting: HashSet::new(),
    };

    // Loop until there are no more unmarked states.
    while s_op.is_some() {
        let s = s_op.unwrap();

        // Split the positions in current state by associated character.
        // Store the union of followpos of that position.
        let mut followpos_split: HashMap<CharType, HashSet<u32>> = HashMap::new();
        s.positions
            .iter()
            .try_for_each(|i| -> Result<(), ParseError> {
                let leaf = base.leaves.get(&i).ok_or(ParseError::Dfa)?;
                let leaf_char = &leaf.character;
                let c = (*leaf_char).as_ref().ok_or(ParseError::Dfa)?;

                let followpos = leaf.followpos.clone();
                match followpos_split.get_mut(&c) {
                    Some(u) => *u = hash_set_union(u, &followpos),
                    None => {
                        followpos_split.insert(c.clone(), followpos);
                    }
                };

                Ok(())
            })?;

        // Create new states based on created unions and update the transition table.
        followpos_split
            .into_iter()
            .try_for_each(|(c, fp_union)| -> Result<(), ParseError> {
                label += 1;
                let mut new_state = DState {
                    label,
                    positions: fp_union,
                };

                // If state does not exist yet, push to end of unmarked states.
                let push_unmarked;
                let in_marked = marked_states
                    .iter()
                    .find(|ms: &&DState| ms.positions == new_state.positions);
                match in_marked {
                    Some(s) => {
                        new_state.label = s.label;
                        push_unmarked = false;
                    }
                    None => {
                        let in_unmarked = unmarked_states
                            .iter()
                            .find(|ums: &&DState| ums.positions == new_state.positions);

                        match in_unmarked {
                            Some(us) => {
                                new_state.label = us.label;
                                push_unmarked = false;
                            }
                            None => {
                                if s.positions == new_state.positions {
                                    new_state.label = s.label;
                                    push_unmarked = false;
                                } else {
                                    push_unmarked = true;
                                }
                            }
                        }
                    }
                };

                // Update the transition table entry.
                if c == CharType::EndMarker {
                    dfa.accepting.insert(s.label);
                } else {
                    dfa.trans.set(s.label, c, new_state.label);
                }

                if push_unmarked {
                    unmarked_states.push_back(new_state);
                }

                Ok(())
            })?;

        // Push current state to handled states.
        marked_states.push(s);
        // Handle the next unmarked state.
        s_op = unmarked_states.pop_front();
    }

    Ok(dfa)
}

#[derive(Clone, Debug)]
struct AugmentedNode {
    character: Option<CharType>,
    accepting: bool,

    nullable: bool,
    firstpos: HashSet<u32>,
    lastpos: HashSet<u32>,
    followpos: HashSet<u32>,
}

fn calculate_functions(tree: &SyntaxTree) -> Result<Option<DFABase>, ParseError> {
    let mut node_lookup = HashMap::new();
    let augmented_ret = match augment_tree(tree, &mut node_lookup, &mut 0)? {
        Some(aug) => aug,
        None => return Ok(None),
    };

    let root_firstpos = match augmented_ret {
        AugmentTreeRet::Leaf(m) => node_lookup.get(&m).ok_or(ParseError::Dfa)?.firstpos.clone(),
        AugmentTreeRet::Branch(n) => n.firstpos,
    };

    Ok(Some(DFABase {
        root_firstpos,
        leaves: node_lookup,
    }))
}

#[derive(Clone, Debug)]
enum AugmentTreeRet {
    Leaf(u32),
    Branch(AugmentedNode),
}

/// Each node is given a marker number, and the functions `nullable`, `firstpos`, `lastpos`,
/// and `followpos` are computed and stored for each node via a depth-first traversal.
/// A lookup map is maintained with all nodes is maintained to compute `followpos` values.
impl AugmentTreeRet {
    fn extract(self, lookup: &mut HashMap<u32, AugmentedNode>) -> (Option<u32>, AugmentedNode) {
        let mark = match self {
            AugmentTreeRet::Leaf(m) => Some(m),
            AugmentTreeRet::Branch(_) => None,
        };

        let val = match self {
            AugmentTreeRet::Leaf(m) => lookup.remove(&m).unwrap(),
            AugmentTreeRet::Branch(aug_n) => aug_n,
        };
        (mark, val)
    }
}

fn augment_tree(
    tree: &SyntaxTree,
    lookup: &mut LeafLookup,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    let augmented = match &tree {
        SyntaxTree::Branch(ref op, ref c1, ref c2) => {
            match op {
                Operator::Kleene => augment_kleene(c1, lookup, mark)?,
                // For alternation node, compute functions for two children to compute `nullable`,
                // `firstpos`, `lastpos`, and `followpos` for this node.
                Operator::Alter => augment_alter(c1, c2, lookup, mark)?,
                // For concat node, compute functions for two children to compute `nullable`,
                // `firstpos`, `lastpos`, and `followpos` for this node.
                Operator::Concat => augment_concat(c1, c2, lookup, mark)?,
            }
        }
        SyntaxTree::Leaf(ty) => augment_leaf(ty, lookup, mark)?,
        SyntaxTree::None => None,
    };

    Ok(augmented)
}

fn augment_kleene(
    c1: &SyntaxTree,
    lookup: &mut LeafLookup,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    // For kleene star node, compute functions for one child to compute `firstpos`,
    // `lastpos`, and `followpos` for this node. Star node is nullable.
    let aug_c1_ret = match augment_tree(c1, lookup, mark)? {
        Some(ret) => ret,
        None => return Ok(None),
    };
    // Remove first child from lookup if is leaf, insert back at the end
    let (aug_c1_mark, mut aug_c1) = aug_c1_ret.extract(lookup);

    let firstpos = aug_c1.firstpos.clone();
    let lastpos = aug_c1.lastpos.clone();

    // All positions in firstpos are in followpos of each position i in lastpos.
    let _ = lastpos.iter().try_for_each(|i| -> Result<(), ParseError> {
        if aug_c1_mark.is_some() && *i == aug_c1_mark.unwrap() {
            aug_c1.followpos = hash_set_union(&aug_c1.followpos, &firstpos);
        } else if let Some(i_pos) = lookup.get_mut(i) {
            i_pos.followpos = hash_set_union(&i_pos.followpos, &firstpos);
        }
        Ok(())
    })?;

    let aug_node = AugmentedNode {
        character: None,
        accepting: false,

        nullable: true,
        firstpos,
        lastpos,
        // followpos is calculated by parent, based on operation type.
        followpos: HashSet::new(),
    };

    reinsert_leaf(lookup, aug_c1_mark, aug_c1);

    Ok(Some(AugmentTreeRet::Branch(aug_node)))
}

fn augment_alter(
    c1: &SyntaxTree,
    c2: &SyntaxTree,
    lookup: &mut LeafLookup,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    let aug_c1_ret = match augment_tree(c1, lookup, mark)? {
        Some(ret) => ret,
        None => return Ok(None),
    };
    // Remove first child from lookup if is leaf, insert back at the end
    let (aug_c1_mark, aug_c1) = aug_c1_ret.extract(lookup);

    // Calculate second child
    let aug_c2_ret = match augment_tree(c2, lookup, mark)? {
        Some(ret) => ret,
        None => {
            reinsert_leaf(lookup, aug_c1_mark, aug_c1);
            return Ok(None);
        }
    };
    // Remove second child from lookup if is leaf, insert back at the end
    let (aug_c2_mark, aug_c2) = aug_c2_ret.extract(lookup);

    let aug_node = AugmentedNode {
        character: None,
        accepting: false,

        // Nullable if one child is nullable.
        nullable: aug_c1.nullable || aug_c2.nullable,
        // firstpos is union of firstpos of children.
        firstpos: hash_set_union(&aug_c1.firstpos, &aug_c2.firstpos),
        // lastpos is union of lastpos of children.
        lastpos: hash_set_union(&aug_c1.lastpos, &aug_c2.lastpos),
        // followpos is calculated by parent, based on operation type.
        followpos: HashSet::new(),
    };

    // Insert children back into lookup if leaf
    reinsert_leaf(lookup, aug_c1_mark, aug_c1);
    reinsert_leaf(lookup, aug_c2_mark, aug_c2);

    Ok(Some(AugmentTreeRet::Branch(aug_node)))
}

fn augment_concat(
    c1: &SyntaxTree,
    c2: &SyntaxTree,
    lookup: &mut LeafLookup,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    let aug_c1_ret = augment_tree(c1, lookup, mark)?;
    if aug_c1_ret.is_none() {
        return augment_tree(c2, lookup, mark);
    }

    let aug_c1_ret = aug_c1_ret.unwrap();
    // Copy to save and return if second child is None; awful hack, but oh well.
    let aug_c1_ret_dup = aug_c1_ret.clone();
    // Remove first child from lookup if is leaf, insert back at the end
    let (aug_c1_mark, mut aug_c1) = aug_c1_ret.extract(lookup);

    // Calculate second child
    let aug_c2_ret = match augment_tree(c2, lookup, mark)? {
        Some(ret) => ret,
        None => {
            reinsert_leaf(lookup, aug_c1_mark, aug_c1);
            return Ok(Some(aug_c1_ret_dup));
        }
    };

    // Remove second child from lookup if is leaf, insert back at the end
    let (aug_c2_mark, aug_c2) = aug_c2_ret.extract(lookup);

    // If first child is nullable, firstpos must also contain firstpos of second
    // child.
    let firstpos = if aug_c1.nullable {
        hash_set_union(&aug_c1.firstpos, &aug_c2.firstpos)
    } else {
        aug_c1.firstpos.clone()
    };

    // If second child is nullable, lastpos must also contain lastpos of first
    // child.
    let lastpos = if aug_c2.nullable {
        hash_set_union(&aug_c1.lastpos, &aug_c2.lastpos)
    } else {
        aug_c2.lastpos.clone()
    };

    // All positions in firstpos of second child are in followpos of every position
    // i in lastpos of first child
    let _ = aug_c1
        .lastpos
        .clone()
        .iter()
        .map(|i| -> Result<(), ParseError> {
            if aug_c1_mark.is_some() && *i == aug_c1_mark.unwrap() {
                aug_c1.followpos = hash_set_union(&aug_c1.followpos, &aug_c2.firstpos)
            } else if let Some(i_pos) = lookup.get_mut(i) {
                i_pos.followpos = hash_set_union(&i_pos.followpos, &aug_c2.firstpos);
            }

            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let aug_node = AugmentedNode {
        character: None,
        accepting: false,

        nullable: aug_c1.nullable && aug_c2.nullable,
        firstpos,
        lastpos,
        // followpos is calculated by parent, based on operation type.
        followpos: HashSet::new(),
    };

    // Reinsert second child
    reinsert_leaf(lookup, aug_c1_mark, aug_c1);
    reinsert_leaf(lookup, aug_c2_mark, aug_c2);
    Ok(Some(AugmentTreeRet::Branch(aug_node)))
}

fn augment_leaf(
    ty: &CharType,
    lookup: &mut LeafLookup,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    *mark += 1;

    let aug_leaf = AugmentedNode {
        character: Some(ty.clone()),
        accepting: false,

        nullable: false,
        firstpos: hash_set![*mark],
        lastpos: hash_set![*mark],
        followpos: HashSet::new(),
    };

    if lookup.insert(*mark, aug_leaf).is_some() {
        return Err(ParseError::Dfa);
    }

    Ok(Some(AugmentTreeRet::Leaf(*mark)))
}

fn reinsert_leaf(lookup: &mut HashMap<u32, AugmentedNode>, mark: Option<u32>, node: AugmentedNode) {
    if let Some(m) = mark {
        lookup.insert(m, node);
    }
}

fn hash_set_union<T: Clone + Eq + std::hash::Hash>(s1: &HashSet<T>, s2: &HashSet<T>) -> HashSet<T> {
    s1.union(s2).cloned().collect()
}
