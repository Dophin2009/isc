# RE

This is an attempt at a simple regular expression parser and matcher.
Valid operators in this regular expression library are, in order of
decreasing precedence, grouping (`()`), the Kleene star (`*`),
concatenation, and alternation (`|`). Character classes, lookaheads, and
other extended features are not supported.

## Method

Regular expressions are compiled by conversion to a syntax tree, then
direct conversion to a determinate finite automaton, as outlined in
Algorithm 3.36 from *Compilers: Principles, Techniques, and Tool*,
Second Edition.

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

### Syntax Tree to Determinate Finite Automaton

A depth-first traversal of the syntax tree is used to compute the
function `followpos`, the set of leaves that can immediately follow the
given one in a string scan, for each leaf. The `followpos` sets are used
to construct the states of the DFA.

The DFA is represented by a structure consisting of three components:

-   the label of the starting state
-   a table to determine, given the current state and the next
    character, which state to advance to
-   a set of labels of the accepting states

### Matching

Whether a string matches the regular expression is determined by
stepping through the DFA by iteration of the string's characters. When
there are no more characters, the set of accepting states is checked for
the presence of the final state. The string is part of the language
described by the regular expression if the final state is an accepting
one; otherwise, it is not.
