use crate::lexer::{Lexer, Rule};

use std::collections::HashMap;

use automata::dfa::{DFAFromNFA, Transition};
use automata::{DFA, NFA};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use regexp2::class::{CharClass, CharRange};
use regexp2::parser::{NFAParser, Parser};
use syn::{parse_macro_input, Expr};

pub fn lexer(tok: TokenStream) -> TokenStream {
    let Lexer {
        vis,
        name,
        str_ident,
        return_type,
        rules,
    } = parse_macro_input!(tok as Lexer);
    let (nfa, action_mapping) = parse_combined_nfa(&rules);
    let DFAFromNFA { dfa, nfa_mapping }: DFAFromNFA<_> = nfa.into();

    let dfa_static = lazy_static_dfa(&dfa);
    (quote! {
        #dfa_static

        #vis fn #name(input: &mut str) -> Option<#return_type> {
            let (m, final_state) = match LEXER_DFA.find(&input.chars()) {
                Some(m) => m,
                None => return None,
            };

            None
        }

    })
    .into()
}

const INVALID_REGEXP_ERROR: &str = "invalid regular expression";

// Parse the rules into a single NFA and a map of final states to action expressions.
fn parse_combined_nfa(rules: &Vec<Rule>) -> (NFA<CharClass>, HashMap<u32, &Expr>) {
    let nfa_parser = NFAParser::new();
    // Parse regular expression strings into NFAs.
    let nfa_sub: Vec<_> = rules
        .into_iter()
        .filter_map(
            |Rule { regexp, action }| match nfa_parser.parse(&regexp.value()) {
                // Throw errors if failed to parse.
                Ok(op) => match op {
                    Some(n) => Some((n, action)),
                    // None returned means error.
                    None => {
                        regexp.span().unstable().error(INVALID_REGEXP_ERROR).emit();
                        None
                    }
                },
                Err(e) => {
                    regexp
                        .span()
                        .unstable()
                        .error(format!("{}: {}", INVALID_REGEXP_ERROR, e))
                        .emit();
                    None
                }
            },
        )
        .collect();

    // Combine NFAs into a single NFA.
    let mut action_mapping = HashMap::new();
    let mut nfa = NFA::new();
    for (sub, action) in nfa_sub.iter() {
        let offset = nfa.total_states;

        NFA::copy_into(&mut nfa, sub);
        // Map new, offsetted final states to their original action.
        for sub_final in sub.final_states.iter() {
            action_mapping.insert(*sub_final + offset, *action);
        }
    }

    (nfa, action_mapping)
}

fn lazy_static_dfa(dfa: &DFA<CharClass>) -> TokenStream2 {
    let initial_state = dfa.initial_state;
    let total_states = dfa.total_states;
    let final_states: Vec<_> = dfa.final_states.iter().collect();
    let transitions_args: Vec<_> = dfa
        .transition
        .clone()
        .into_iter()
        .map(|(src, Transition(tr), dest)| {
            let ranges: Vec<_> = tr
                .ranges
                .iter()
                .map(|CharRange { start, end }| quote! { CharRange::new(#start, #end) })
                .collect();
            quote! { #src, [ #( CharClass::new(#ranges) ),* ], #dest }
        })
        .collect();

    quote! {
        use std::collections::HashSet;

        use automata::{DFA, table::Table};
        use regexp2::class::{CharClass, CharRange};
        use lazy_static::lazy_static;

        lazy_static! {
            static ref LEXER_DFA: DFA<CharClass> = {
                let mut dfa = DFA::new();
                dfa.initial_state = #initial_state;
                dfa.total_states = #total_states;
                dfa.final_states = HashSet::new();
                dfa.final_states.extend(&[ #( #final_states ),* ]);

                dfa.transition = Table::new();
                #( dfa.transition.set(#transitions_args) )*

                dfa
            };
        }
    }
}
