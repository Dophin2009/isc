use super::ast::{LeafType, Operator, SyntaxTree};
use super::error::ParseError;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct DFA {
    pub start: u32,
    pub trans: HashMap<u32, HashMap<char, u32>>,
}

/// Implements steps 2 and 3 of **Algorithm 3.36** in *Compilers: Principles,
/// Techniques, and Tool*, Second Edition.
pub fn tree_to_dfa(tree: &SyntaxTree) -> Result<(), ParseError> {
    let _augmented = calculate_functions(tree)?;
    Ok(())
}

#[derive(Clone, Debug)]
struct AugmentedNode {
    character: Option<char>,

    nullable: bool,
    firstpos: HashSet<u32>,
    lastpos: HashSet<u32>,
    followpos: HashSet<u32>,
}

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

#[derive(Debug)]
struct DFABase {
    root_firstpos: HashSet<u32>,
    followpos: HashMap<u32, AugmentedNode>,
}

fn calculate_functions(tree: &SyntaxTree) -> Result<DFABase, ParseError> {
    let mut node_lookup = HashMap::new();
    let augmented_ret = augment_tree(tree, &mut node_lookup, &mut 0)?.unwrap(); // Fix error handling
    let root_firstpos = match augmented_ret {
        AugmentTreeRet::Leaf(m) => node_lookup
            .get(&m)
            .ok_or(ParseError::Malformed)?
            .firstpos
            .clone(),
        AugmentTreeRet::Branch(n) => n.firstpos,
    };

    Ok(DFABase {
        root_firstpos,
        followpos: node_lookup,
    })
}

enum AugmentTreeRet {
    Leaf(u32),
    Branch(AugmentedNode),
}

/// Each node is given a marker number, and the functions `nullable`, `firstpos`, `lastpos`,
/// and `followpos` are computed and stored for each node via a depth-first traversal.
/// A lookup map is maintained with all nodes is maintained to compute `followpos` values.
impl AugmentTreeRet {
    fn extract(
        self,
        lookup: &mut HashMap<u32, AugmentedNode>,
    ) -> Result<(Option<u32>, AugmentedNode), ParseError> {
        let mark = match self {
            AugmentTreeRet::Leaf(m) => Some(m),
            AugmentTreeRet::Branch(_) => None,
        };

        let val = match self {
            AugmentTreeRet::Leaf(m) => lookup.remove(&m).ok_or(ParseError::Malformed)?,
            AugmentTreeRet::Branch(aug_n) => aug_n,
        };
        Ok((mark, val))
    }
}

fn augment_tree<'a>(
    tree: &SyntaxTree,
    lookup: &mut HashMap<u32, AugmentedNode>,
    mark: &mut u32,
) -> Result<Option<AugmentTreeRet>, ParseError> {
    let augmented = match &tree {
        SyntaxTree::Branch(ref op, ref c1, ref c2) => {
            // Calculate first child
            let aug_c1_ret = augment_tree(c1, lookup, mark)?.ok_or(ParseError::Malformed)?;
            // Remove first child from lookup if is leaf, insert back at the end
            let (aug_c1_mark, aug_c1) = aug_c1_ret.extract(lookup)?;

            let aug_node = match op {
                // For kleene star node, compute functions for one child to compute `firstpos`,
                // `lastpos`, and `followpos` for this node. Star node is nullable.
                Operator::Kleene => {
                    let firstpos = aug_c1.firstpos.clone();
                    let lastpos = aug_c1.lastpos.clone();

                    // All positions in firstpos are in followpos of each position i in lastpos.
                    let _ = lastpos
                        .iter()
                        .map(|i| -> Result<(), ParseError> {
                            let i_pos = lookup.get_mut(i).ok_or(ParseError::Malformed)?;
                            i_pos.followpos = hash_set_union(&i_pos.followpos, &firstpos);
                            Ok(())
                        })
                        .collect::<Result<(), _>>()?;

                    let aug_node = AugmentedNode {
                        character: None,

                        nullable: true,
                        firstpos,
                        lastpos,
                        // followpos is calculated by parent, based on operation type.
                        followpos: HashSet::new(),
                    };

                    AugmentTreeRet::Branch(aug_node)
                }
                // For alternation node, compute functions for two children to compute `nullable`,
                // `firstpos`, `lastpos`, and `followpos` for this node.
                Operator::Alter => {
                    // Calculate second child
                    let aug_c2_ret =
                        augment_tree(c2, lookup, mark)?.ok_or(ParseError::Malformed)?;
                    // Remove second child from lookup if is leaf, insert back at the end
                    let (aug_c2_mark, aug_c2) = aug_c2_ret.extract(lookup)?;

                    let aug_node = AugmentedNode {
                        character: None,

                        // Nullable if one child is nullable.
                        nullable: aug_c1.nullable || aug_c2.nullable,
                        // firstpos is union of firstpos of children.
                        firstpos: hash_set_union(&aug_c1.firstpos, &aug_c2.firstpos),
                        // lastpos is union of lastpos of children.
                        lastpos: hash_set_union(&aug_c1.lastpos, &aug_c2.lastpos),
                        // followpos is calculated by parent, based on operation type.
                        followpos: HashSet::new(),
                    };

                    // Insert second child back into lookup if leaf
                    reinsert_leaf(lookup, aug_c2_mark, aug_c2);

                    AugmentTreeRet::Branch(aug_node)
                }
                // For concat node, compute functions for two children to compute `nullable`,
                // `firstpos`, `lastpos`, and `followpos` for this node.
                Operator::Concat => {
                    // Calculate second child
                    let aug_c2_ret =
                        augment_tree(c2, lookup, mark)?.ok_or(ParseError::Malformed)?;
                    // Remove second child from lookup if is leaf, insert back at the end
                    let (aug_c2_mark, aug_c2) = aug_c2_ret.extract(lookup)?;

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
                        .iter()
                        .map(|i| -> Result<(), ParseError> {
                            let i_pos = lookup.get_mut(i).ok_or(ParseError::Malformed)?;
                            i_pos.followpos = hash_set_union(&i_pos.followpos, &aug_c2.lastpos);
                            Ok(())
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let aug_node = AugmentedNode {
                        character: None,

                        nullable: aug_c1.nullable && aug_c2.nullable,
                        firstpos,
                        lastpos,
                        // followpos is calculated by parent, based on operation type.
                        followpos: HashSet::new(),
                    };

                    // Reinsert second child
                    reinsert_leaf(lookup, aug_c2_mark, aug_c2);

                    AugmentTreeRet::Branch(aug_node)
                }
            };

            reinsert_leaf(lookup, aug_c1_mark, aug_c1);

            aug_node
        }
        SyntaxTree::Leaf(ty) => {
            *mark += 1;

            let aug_leaf = augment_leaf(ty, mark);
            match lookup.insert(*mark, aug_leaf) {
                Some(_) => return Err(ParseError::Malformed),
                None => {}
            };
            AugmentTreeRet::Leaf(*mark)
        }
        SyntaxTree::None => return Ok(None),
    };

    Ok(Some(augmented))
}

fn augment_leaf(ty: &LeafType, mark: &mut u32) -> AugmentedNode {
    let c = match ty {
        // Non-epsilon leaf is:
        //  -  nullable: false
        //  -  firstpos: { mark },
        //  -  lastpos: { mark },
        //  -  followpos:
        LeafType::Char(ch) => *ch,
        LeafType::Whitespace => ' ',
        LeafType::Newline => '\n',
    };

    AugmentedNode {
        character: Some(c),

        nullable: false,
        firstpos: hash_set![*mark],
        lastpos: hash_set![*mark],
        followpos: HashSet::new(),
    }
}

fn reinsert_leaf(lookup: &mut HashMap<u32, AugmentedNode>, mark: Option<u32>, node: AugmentedNode) {
    match mark {
        Some(m) => {
            lookup.insert(m, node);
        }
        None => {}
    }
}

fn hash_set_union<T: Clone + Eq + std::hash::Hash>(s1: &HashSet<T>, s2: &HashSet<T>) -> HashSet<T> {
    s1.union(s2).map(|u| u.clone()).collect()
}
