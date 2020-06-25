use super::ParseError;
use crate::convert::CharType;

#[derive(Debug, PartialEq)]
pub enum Node<T, U> {
    Leaf(T),
    Branch(U, Box<Node<T, U>>, Box<Node<T, U>>),
    None,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
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

pub type SyntaxTree = Node<CharType, Operator>;

pub fn syntax_tree(expr: &str) -> Result<SyntaxTree, ParseError> {
    let mut op_stack = Vec::new();
    let mut node_stack = Vec::new();
    let mut paren_count_stack = Vec::new();

    let mut insert_concat = false;
    for c in expr.chars() {
        match c {
            // Kleene star; push kleene operator to the op stack.
            // Highest precedence operator; does not collapse operators to the left unless another
            // kleene star is encountered; next step will collapse this operator and top-most node
            // into node immediately.
            // Operates only on node to the left; concat is inserted after if non-meta-char or left
            // parenthesis is encountered.
            '*' => {
                let op = OperatorFlag::Kleene;
                precedence_collapse_stack(&op, &mut node_stack, &mut op_stack)?;

                op_stack.push(op);
                insert_concat = true;
            }
            // Alternation operator; push alternation operator to the op stack.
            // Lowest precedence operator; will immediately collapse previous operator.
            // Operates on node to left and right; will not collapse until the end.
            '|' => {
                let op = OperatorFlag::Alter;
                precedence_collapse_stack(&op, &mut node_stack, &mut op_stack)?;

                op_stack.push(op);
                insert_concat = false;
            }
            // Left parentheses; push parentheses to stack as segment begin marker.
            // Collapse stack once if previous operator was a kleene star; this grouped segment
            // could be the right branch of a concatenation or alternation.
            // When next right parenthesis is encountered; entire segment will be collapsed. All
            // but lowest precedence operators should already be collapsed at that point.
            '(' => {
                let op = OperatorFlag::LeftParen;
                precedence_collapse_stack(&op, &mut node_stack, &mut op_stack)?;

                // Insert concat operator before this operator if previous token was kleene star,
                // non-meta char, or right parenthesis.
                if insert_concat {
                    op_stack.push(OperatorFlag::Concat);
                }

                // Push segment begin marker.
                op_stack.push(op);

                // Store the current number of nodes on the stack; if the contents of these
                // parenthesis are empty (), then the number of nodes should not change between now
                // and the occurrence of the right parenthesis.
                paren_count_stack.push(node_stack.len());

                insert_concat = false;
            }
            // Right parenthesis; collapse stack until next operator on stack is a left
            // parenthesis.
            // Collapse the stack until the left parenthesis is found on the stack.
            // A concat should be inserted after if the next token is a non-meta-char, or
            // left parenthesis.
            ')' => {
                // If parenthesis was empty (), insert a None node;
                let last_op = op_stack.last().ok_or(ParseError::Malformed)?;
                let prev_node_count = paren_count_stack.last().ok_or(ParseError::Malformed)?;
                if *last_op == OperatorFlag::LeftParen && *prev_node_count == node_stack.len() {
                    op_stack.pop().ok_or(ParseError::Malformed)?;
                    node_stack.push(Node::None);
                } else {
                    while !op_stack.is_empty()
                        && *op_stack.last().unwrap() != OperatorFlag::LeftParen
                    {
                        collapse_stack(&mut node_stack, &mut op_stack)?;
                    }
                    // Pop the left parenthesis off the op stack.
                    op_stack.pop().ok_or(ParseError::Malformed)?;
                }
                insert_concat = true;
            }
            // Any other character; considered a non-meta-char.
            // Kleene star and concat operators before this should be collapsed.
            // If after a kleene star, right parenthesis, or other non-meta-char, a concat operator
            // should be inserted.
            _ => {
                while precedence_collapse_stack(
                    &OperatorFlag::Concat,
                    &mut node_stack,
                    &mut op_stack,
                )? {}

                // Insert a concat operator between two non-meta-chars.
                if insert_concat {
                    op_stack.push(OperatorFlag::Concat);
                }

                let leaf_type = c.into();
                node_stack.push(Node::Leaf(leaf_type));
                insert_concat = true;
            }
        }
        // println!("{:?}", node_stack);
        // println!("{:?}\n", op_stack);
    }

    // Collapse the stack until no more operators remain
    while !op_stack.is_empty() {
        collapse_stack(&mut node_stack, &mut op_stack)?;
    }

    let head: Node<_, _> = node_stack.into_iter().last().unwrap_or(Node::None);
    let root = Node::Branch(
        Operator::Concat,
        Box::new(head),
        Box::new(Node::Leaf(CharType::EndMarker)),
    );

    Ok(root)
}

/// Collapses the stack if the current operator `new_op` has equal or lower precedence than the last operator.
fn precedence_collapse_stack(
    new_op: &OperatorFlag,
    nodes: &mut Vec<Node<CharType, Operator>>,
    ops: &mut Vec<OperatorFlag>,
) -> Result<bool, ParseError> {
    let collapse = match ops.last() {
        Some(last_op) => {
            if last_op == new_op && *last_op != OperatorFlag::LeftParen {
                // If current op is the same as last, collapse the last.
                // If both of left parenthesis, do nothing
                true
            } else if *new_op == OperatorFlag::Alter {
                // If current op is alternation, collapse last if it is kleene or concat.
                *last_op == OperatorFlag::Kleene || *last_op == OperatorFlag::Concat
            } else if *new_op == OperatorFlag::Concat {
                // If current op is concat, collapse last if it is kleene star.
                *last_op == OperatorFlag::Kleene
            } else if *new_op == OperatorFlag::Kleene {
                // If current op is kleene star, do not collapse last because kleene star is
                // highest precedence.
                false
            } else if *new_op == OperatorFlag::LeftParen {
                // If current op is left parenthesis, collapse last if it is kleene star.
                // Kleene star operates only on left node.
                *last_op == OperatorFlag::Kleene || *last_op == OperatorFlag::Concat
            } else {
                false
            }
        }
        None => false,
    };

    if collapse {
        collapse_stack(nodes, ops)?;
    }

    Ok(collapse)
}

/// Take the top operator and construct a new node with children from the node stack.
fn collapse_stack(
    nodes: &mut Vec<Node<CharType, Operator>>,
    ops: &mut Vec<OperatorFlag>,
) -> Result<(), ParseError> {
    let op = ops.pop().ok_or(ParseError::Malformed)?;

    let n_op: Operator;
    let c1: Node<_, _>;
    let c2: Node<_, _>;
    match op {
        // Kleene star op constructs a branch with one child.
        OperatorFlag::Kleene => {
            n_op = Operator::Kleene;
            c1 = nodes.pop().ok_or(ParseError::Malformed)?;
            c2 = Node::None;
        }
        // Alternation op constructs two child branch.
        OperatorFlag::Alter => {
            n_op = Operator::Alter;
            c2 = nodes.pop().ok_or(ParseError::Malformed)?;
            c1 = nodes.pop().ok_or(ParseError::Malformed)?;
        }
        // Concatenation op constructs two child branch.
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
