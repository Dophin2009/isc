#![deny(rust_2018_idioms)]
#![deny(future_incompatible)]

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use regexp2::{
    automata::{
        dfa::{DFAFromNFA, Transition},
        DFA, NFA,
    },
    class::{CharClass, CharRange},
    parser::{NFAParser, Parser},
};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Expr, Ident, LitStr, Token, Type, Visibility,
};

#[proc_macro]
pub fn lexer(tok: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(tok as Lexer);
    match lexer_(parsed) {
        Ok(res) => res.into(),
        Err(res) => res.into(),
    }
}

fn lexer_(parsed: Lexer) -> Result<TokenStream, TokenStream> {
    let Lexer {
        struct_vis,
        struct_name,
        fn_vis,
        fn_name,
        span_id,
        return_type,
        error_variant,
        rules,
    } = parsed;

    let (nfa, action_mapping) = parse_combined_nfa(&rules)?;
    let DFAFromNFA { dfa, nfa_mapping }: DFAFromNFA<_> = nfa.into();

    let dfa_rebuilt = dfa_rebuilt(&dfa);

    let dfa_actions: Vec<_> = nfa_mapping
        .iter()
        .filter_map(|(dfa_state, nfa_states)| {
            action_mapping
                .iter()
                .filter(|(nfa_state, _)| nfa_states.contains(nfa_state))
                .min_by_key(|(_, (_, precedence))| precedence)
                .map(|(_, (action, _))| (dfa_state, action))
        })
        .collect();

    let action_fns: Vec<_> = dfa_actions
        .iter()
        .map(|(dfa_state, action)| {
            let fn_name = format_ident!("action_{}", dfa_state);
            quote! {
                #[allow(unused)]
                #[allow(clippy::unnecessary_wraps)]
                #[inline]
                fn #fn_name(#span_id: &str) -> std::option::Option<#return_type> {
                    #action
                }
            }
        })
        .collect();

    let action_match: Vec<_> = dfa_actions
        .iter()
        .map(|(dfa_state, _)| {
            let fn_call = format_ident!("action_{}", dfa_state);
            quote!(#dfa_state => #fn_call(&span))
        })
        .collect();

    Ok(quote! {
        #[derive(Debug, Clone)]
        #struct_vis struct #struct_name {
            dfa: ::llex::stream::LexerDFA,
        }

        impl #struct_name {
            #[inline]
            #struct_vis fn new() -> Self {
                let dfa = #dfa_rebuilt;
                Self { dfa }
            }

            #[inline]
            #fn_vis fn #fn_name<'a, I>(&self, input: I) -> ::llex::LexerStream<#return_type, &#struct_name, I>
            where
                I: std::iter::Iterator<Item = char>,
            {
                ::llex::LexerStream::new(self, input)
            }
        }

        impl ::llex::stream::LexerDFAMatcher<#return_type> for #struct_name {
            #[inline]
            fn tokenize<'a, I>(&self, input: &mut std::iter::Peekable<I>) -> std::option::Option<(#return_type, ::llex::regexp2::automata::Match<char>)>
            where
                I: std::iter::Iterator<Item = char>,
            {
                #(
                    #action_fns
                )*

                // Step through DFA to the find the longest match.
                let (m, final_state) = match self.dfa.find_mut(input) {
                    std::option::Option::Some(m) => m,
                    std::option::Option::None => {
                        input.next();
                        return std::option::Option::Some((#error_variant, ::llex::regexp2::automata::Match::new(0, 0, vec![])));
                    },
                };

                // Execute the action expression corresponding to the final state.
                let span: std::string::String = m.span.iter().cloned().collect();
                let token_op = match final_state {
                    #( #action_match ),*,
                    // Catch-all branch should never execute?
                    _ => std::unreachable!(),
                };

                token_op.map(|t| (t, m))
            }
        }

        impl ::llex::stream::LexerDFAMatcher<#return_type> for &#struct_name {
            #[inline]
            fn tokenize<I>(&self, input: &mut std::iter::Peekable<I>) -> std::option::Option<(#return_type, ::llex::regexp2::automata::Match<char>)>
            where
                I: std::iter::Iterator<Item = char>,
            {
                (*self).tokenize(input)
            }
        }

    })
}

struct Lexer {
    struct_vis: Option<Visibility>,
    struct_name: Ident,
    fn_vis: Option<Visibility>,
    fn_name: Ident,

    span_id: Ident,
    return_type: Type,
    error_variant: Expr,

    rules: Vec<Rule>,
}

impl Parse for Lexer {
    #[inline]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        macro_rules! token {
            ($x:tt) => {
                input.parse::<Token![$x]>()?
            };
        }

        let struct_vis = input.parse().ok();
        token!(struct);
        let struct_name = input.parse()?;
        token!(;);

        let fn_vis = input.parse().ok();
        token!(fn);
        let fn_name = input.parse()?;
        token!(;);

        let span_id = {
            let inner;
            parenthesized!(inner in input);
            let span_id = inner.parse()?;
            if !inner.is_empty() {
                return Err(inner.error("Unexpected token after token string identifier"));
            }
            span_id
        };

        token!(->);
        let return_type = input.parse()?;
        token!(,);

        let error_variant = input.parse()?;
        token!(;);

        let rules = {
            let mut rules = Vec::new();
            while !input.is_empty() {
                let regexp = input.parse()?;
                input.parse::<Token![=>]>()?;

                let optional_comma = input.peek(token::Brace);

                let action = input.parse()?;
                let rule = Rule::new(regexp, action);

                match input.parse::<Token![,]>() {
                    Ok(_) => {}
                    Err(e) => {
                        if !input.is_empty() && !optional_comma {
                            return Err(e);
                        }
                    }
                }

                rules.push(rule);
            }
            rules
        };

        Ok(Self {
            struct_vis,
            struct_name,
            fn_vis,
            fn_name,
            span_id,
            return_type,
            error_variant,
            rules,
        })
    }
}

struct Rule {
    regexp: LitStr,
    action: Expr,
}

impl Rule {
    #[inline]
    fn new(regexp: LitStr, action: Expr) -> Self {
        Self { regexp, action }
    }
}

const INVALID_REGEXP_ERROR: &str = "invalid regular expression";

// Parse the rules into a single NFA and a map of final states to action expressions.
#[inline]
#[allow(clippy::type_complexity)]
fn parse_combined_nfa(
    rules: &[Rule],
) -> Result<(NFA<CharClass>, HashMap<usize, (&Expr, usize)>), TokenStream> {
    let nfa_parser = NFAParser::new();
    // Parse regular expression strings into NFAs.
    let nfa_sub: Vec<_> = rules
        .iter()
        .map(
            |Rule { regexp, action }| match nfa_parser.parse(&regexp.value()) {
                // Throw errors if failed to parse.
                Ok(op) => match op {
                    Some(n) => Ok(Some((n, action))),
                    // None returned means error.
                    None => Err(span_error(regexp.span(), INVALID_REGEXP_ERROR)),
                },
                Err(e) => Err(span_error(
                    regexp.span(),
                    &format!("{}: {}", INVALID_REGEXP_ERROR, e),
                )),
            },
        )
        .collect::<Result<_, _>>()?;

    let nfa_sub: Vec<_> = nfa_sub.into_iter().filter_map(|nfa| nfa).collect();

    // Combine NFAs into a single NFA.
    let mut action_mapping = HashMap::new();
    let mut nfa = NFA::new();
    let mut offset = nfa.total_states;
    for (precedence, (sub, action)) in nfa_sub.iter().enumerate() {
        NFA::copy_into(&mut nfa, sub);
        nfa.add_epsilon_transition(nfa.initial_state, sub.initial_state + offset);
        // Map new, offsetted final states to their original action.
        for sub_final in sub.final_states.iter() {
            nfa.final_states.insert(*sub_final + offset);
            action_mapping.insert(*sub_final + offset, (*action, precedence));
        }

        offset += sub.total_states;
    }

    Ok((nfa, action_mapping))
}

fn dfa_rebuilt(dfa: &DFA<CharClass>) -> TokenStream {
    let initial_state = dfa.initial_state;
    let total_states = dfa.total_states;
    let final_states: Vec<_> = dfa.final_states.iter().collect();
    let transition_sets: Vec<_> = dfa
        .transition
        .clone()
        .into_iter()
        .map(|(src, Transition(tr), dest)| {
            let ranges: Vec<_> = tr
                .ranges
                .iter()
                .map(|CharRange { start, end }| quote!(::llex::regexp2::class::CharRange::new(#start, #end)))
                .collect();
            quote! {
                dfa.transition.set(#src, ::llex::regexp2::automata::dfa::Transition(vec![ #( #ranges ),* ].into()), #dest);
            }
        })
        .collect();

    quote! {
        {
            let mut dfa = ::llex::regexp2::automata::DFA::new();
            dfa.initial_state = #initial_state;
            dfa.total_states = #total_states;
            dfa.final_states = std::collections::HashSet::new();
            dfa.final_states.extend(&[ #( #final_states ),* ]);

            dfa.transition = ::llex::regexp2::automata::table::Table::new();
            #( #transition_sets )*

            dfa
        }
    }
}

fn span_error(span: Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}
