# RE

This is an attempt at a simple regular expression parser and matcher.
Valid operators in this regular expression library are, in order of
decreasing precedence, grouping (`()`), the Kleene star (`*`),
concatenation, and alternation (`|`). Character classes, lookaheads, and
other extended features are not supported.

## Method

Regular expressions are compiled by conversion to a syntax tree, then
direct conversion to a DFA, as outlined in Algorithm 3.36 from
*Compilers: Principles, Techniques, and Tool*, Second Edition.

### Expression to Syntax Tree

The syntax tree is constructed during one pass of the expression string.

Two stacks are maintained, `nodes` and `ops`, which contain tree nodes
and operations, respectively. When meta-characters such as `*` or `(`
are encountered, they are pushed to `ops`. Concatenation operators are
pushed as needed. Non-meta-characters are pushed as leaves to `nodes`.

At each step, nodes are popped off the stack based on the current
operator and the top-most operator on the stack, and a new combined node
with those previous nodes as children is pushed. The overall order of
this collapsing depends on the precedence on the operators.

When reading of the expression string is complete, the nodes are
collapsed together until a single node remains in `nodes`. This node
formed the completed syntax tree.
