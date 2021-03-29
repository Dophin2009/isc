# Yacc Examples

This directory contains a couple of Yacc and Lex examples to try out
those tools. The examples are taken from *Compilers: Principles,
Techniques, and Tool*, Second Edition.

## Calculator

See [calculator](./calculator).

A basic four-function calculator, from Example 4.70.

    1 + 1
    2

    2 * 3
    6

    (4 * 5 + 1) / 3
    7

## Boolean Evaluator

See [bool-eval](./bool-eval).

A program that takes boolean expressions as input and evaluates the
truth value, from Exercise 4.9.1.

    true && false
    false

    true || false
    true

    (true || false) && true
    true
