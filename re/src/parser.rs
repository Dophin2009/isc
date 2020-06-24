use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum Node<T, U> {
    Leaf(T),
    Branch(U, Box<Node<T, U>>, Box<Node<T, U>>),
    None,
}

#[derive(Debug, PartialEq)]
enum LeafType {
    Char(char),
    Newline,
    Whitespace,
}

#[derive(Debug, PartialEq)]
enum Operator {
    Kleene,
    Concat,
    Alter,
}

#[derive(Debug, PartialEq)]
enum OperatorFlag {
    Kleene,
    Concat,
    Alter,
    LeftParen,
}

type SyntaxTree = Node<LeafType, Operator>;

/// This function attempts to implement **Algorithm 3.36**, the conversion of a regular expression
/// string directly to a DFA, from *Compilers: Principles, Techniques, and Tool*, Second Edition.
pub fn regex_to_dfa(expr: &str) -> Result<(), ParseError> {
    let _ast = syntax_tree(expr)?;
    Ok(())
}

fn syntax_tree(expr: &str) -> Result<SyntaxTree, ParseError> {
    let mut op_stack = Vec::new();
    let mut node_stack = Vec::new();

    let mut insert_concat = false;
    for c in expr.chars() {
        match c {
            '*' => {
                op_stack.push(OperatorFlag::Kleene);
                insert_concat = true;
            }
            '|' => {
                op_stack.push(OperatorFlag::Alter);
                insert_concat = false;
            }
            '(' => {
                op_stack.push(OperatorFlag::LeftParen);
                if !node_stack.is_empty() {
                    insert_concat = true;
                }
            }
            ')' => {
                while !op_stack.is_empty() && *op_stack.last().unwrap() != OperatorFlag::LeftParen {
                    collapse_stack(&mut node_stack, &mut op_stack)?;
                }
                op_stack.pop().ok_or(ParseError::Malformed)?;
                insert_concat = true;
            }
            _ => {
                while !op_stack.is_empty() && {
                    let last_op = op_stack.last().ok_or(ParseError::Malformed)?;
                    *last_op == OperatorFlag::Kleene || *last_op == OperatorFlag::Concat
                } {
                    collapse_stack(&mut node_stack, &mut op_stack)?;
                }

                // insert a concat operator between two non-meta-chars
                if insert_concat {
                    op_stack.push(OperatorFlag::Concat);
                }

                let leaf_type = match c {
                    ' ' => LeafType::Whitespace,
                    '\n' => LeafType::Newline,
                    _ => LeafType::Char(c),
                };
                node_stack.push(Node::Leaf(leaf_type));
                insert_concat = true;
            }
        }
    }

    while !op_stack.is_empty() {
        collapse_stack(&mut node_stack, &mut op_stack)?;
    }

    let head: Node<_, _> = node_stack.into_iter().last().unwrap_or(Node::None);
    Ok(head)
}

fn collapse_stack(
    nodes: &mut Vec<Node<LeafType, Operator>>,
    ops: &mut Vec<OperatorFlag>,
) -> Result<(), ParseError> {
    let op = ops.pop().ok_or(ParseError::Malformed)?;

    let n_op: Operator;
    let c1: Node<_, _>;
    let c2: Node<_, _>;
    match op {
        OperatorFlag::Kleene => {
            n_op = Operator::Kleene;
            c1 = nodes.pop().ok_or(ParseError::Malformed)?;
            c2 = Node::None;
        }
        OperatorFlag::Alter => {
            n_op = Operator::Alter;
            c2 = nodes.pop().ok_or(ParseError::Malformed)?;
            c1 = nodes.pop().ok_or(ParseError::Malformed)?;
        }
        OperatorFlag::Concat => {
            n_op = Operator::Concat;
            c2 = nodes.pop().ok_or(ParseError::Malformed)?;
            c1 = nodes.pop().ok_or(ParseError::Malformed)?;
        }
        _ => return Err(ParseError::Malformed),
    }

    let new = Node::Branch(n_op, Box::new(c1), Box::new(c2));
    nodes.push(new);

    Ok(())
}

#[derive(Debug)]
pub enum ParseError {
    Malformed,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Malformed => write!(f, "malformed expression"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::Malformed => None,
        }
    }
}
