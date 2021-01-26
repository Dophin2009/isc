# Independent Study on Compilers

This repository contains the materials for my independent study course on
compilers in my senior year of high school. The goal is to write a basic
compiler for a basic, unnamed language.

## Components

The following components have been / are being implemented:

-   [`automata`](./automata) - Implementation of nondeterminstic and
    deterministic finite automata.

-   [`lalr`](./lalr) - Generation of SLR(1), LR(1), and LALR(1) parse tables
    from context-free grammars.

-   [`lalrgen`](./lalrgen) - Parser generator as a procedural macro.

-   [`llex`](./llex) - Lexical analyzer generator as a procedural macro.

-   [`regexp`](./regexp) - Implementation of limited regular expressions.

-   [`regexp2`](./regexp2) - Better implementation of regular expressions with
    support for character classes and other operators.

## Building

The following dependencies are required to be present to build this project:

-   Rust (nightly)

Building the Yacc examples (these are included in POSIX systems):

-   Yacc
-   Lex
