use crate::parse::DestructureType;
use crate::{NonterminalReference, TerminalRefname};

use std::collections::HashMap;

use lalr::lr1::{LR1Action, LR1Table};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

#[inline]
pub(crate) fn codegen<'g>(
    table: LR1Table<'g, usize, usize, (i32, TokenStream)>,
    parser_visibility: Option<Visibility>,
    parser_name: Ident,
    parser_return_type: Type,
    start_rule_lhs: Ident,
    terminal_type: Type,
    grammar_nonterminals: HashMap<Ident, NonterminalReference>,
    grammar_terminals: HashMap<Ident, (usize, TerminalRefname)>,
) -> TokenStream {
    let parser_decl = parser_decl(&parser_visibility, &parser_name);
    let payload_enum_decl = payload_enum_decl(&grammar_nonterminals);
    let map_token_decl = map_token_decl(&terminal_type, &grammar_terminals);
    let get_goto_decl = get_goto_decl(&table);

    let action_match_branches: Vec<_> = table
        .states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let branches: Vec<_> = state
                .actions
                .iter()
                .map(|(terminal, action)| {
                    let action_code = tokenize_action(action);
                    quote! { #terminal => { #action_code } }
                })
                .collect();

            quote! {
                #i => match token_n {
                    #(#branches),*
                    _ => std::panic!("unexpected token"),
                }
            }
        })
        .collect();

    let action_match = quote! {
        match current_state {
            #(#action_match_branches),*
            _ => std::unreachable!(),
        }
    };

    let endmarker_match_branches: Vec<_> = table
        .states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let code = match state.endmarker {
                Some(ref action) => tokenize_action(action),
                // TODO: Handle no action (error)
                None => quote! { panic!() },
            };

            quote! { #i => { #code } }
        })
        .collect();

    let endmarker_match = quote! {
        match current_state {
            #(#endmarker_match_branches),*
            _ => std::panic!("unexpected token"),
        }
    };

    quote! {
        #parser_decl

        impl #parser_name {
            #parser_visibility fn new() -> Self {
                Self {}
            }

            #parser_visibility fn parse<I>(&self, mut input: I) -> Result<#parser_return_type, ()>
            where
                I: Iterator<Item = #terminal_type>,
            {
                #payload_enum_decl
                #map_token_decl

                fn shift(
                    stack: &mut Vec<(usize, Option<#terminal_type>, Option<PayloadNonterminal>)>,
                    new_state: usize,
                    payload: Option<#terminal_type>
                ) {
                    stack.push((new_state, payload, None));
                }

                #get_goto_decl

                let mut stack = Vec::new();
                let mut current_state = 0;
                stack.push((current_state, None, None));

                let mut saved_input: Option<Option<(#terminal_type, usize)>> = None;

                while true {
                    current_state = stack.last().unwrap().0;

                    let (next_token, next_token_n) = match saved_input {
                        Some(saved) => match saved {
                            Some(tup) => (Some(tup.0), Some(tup.1)),
                            None => (None, None),
                        }
                        None => {
                            let next_token = input.next();
                            let next_token_n = match next_token {
                                Some(ref token) => map_token(token),
                                None => None,
                            };
                            (next_token, next_token_n)
                        }
                    };
                    saved_input = None;

                    match next_token {
                        Some(_) => {
                            let token_n = next_token_n.unwrap();
                            #action_match
                        }
                        None => {
                            #endmarker_match
                        }
                    }
                }

                let final_payload = stack.pop().unwrap().2.unwrap();
                let result = match final_payload {
                    PayloadNonterminal::#start_rule_lhs(x) => x,
                    _ => std::unreachable!(),
                };

                Ok(result)
            }
        }
    }
}

#[inline]
fn parser_decl(visibility: &Option<Visibility>, name: &Ident) -> TokenStream {
    quote! {
        #visibility struct #name {}
    }
}

#[inline]
fn payload_enum_decl(grammar_nonterminals: &HashMap<Ident, NonterminalReference>) -> TokenStream {
    let payload_enum_variants: Vec<_> = grammar_nonterminals
        .iter()
        .map(|(ident, reference)| (ident, reference.return_type.clone()))
        .map(|(ident, ty)| quote! { #ident(#ty) })
        .collect();

    quote! {
        #[derive(Debug)]
        enum PayloadNonterminal {
            #(#payload_enum_variants),*
        }
    }
}

#[inline]
fn map_token_decl(
    terminal_type: &Type,
    grammar_terminals: &HashMap<Ident, (usize, TerminalRefname)>,
) -> TokenStream {
    let terminal_map_branches: Vec<_> = grammar_terminals
        .into_iter()
        .map(|(variant, (n, refname))| {
            let variant = match refname {
                TerminalRefname::Destructure(_, ty, _) => {
                    // Duplicate code but oh well
                    match ty {
                        DestructureType::Struct => quote! { #variant { .. } },
                        DestructureType::TupleStruct => quote! { #variant( .. ) },
                    }
                }
                TerminalRefname::Ignore => quote! { #variant },
            };
            quote! { #terminal_type::#variant => Some(#n) }
        })
        .collect();

    quote! {
        fn map_token(token: &#terminal_type) -> Option<usize> {
            match token {
                #(#terminal_map_branches),*,
                _ => std::panic!("unrecognized terminal"),
            }
        }
    }
}

#[inline]
fn get_goto_decl<'g>(table: &LR1Table<'g, usize, usize, (i32, TokenStream)>) -> TokenStream {
    let get_goto_branches: Vec<_> = table
        .states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let branches: Vec<_> = state
                .goto
                .iter()
                .map(|(n, dest)| quote! { #n => { #dest } })
                .collect();

            quote! { #i => match nonterminal {
                #(#branches),*
                _ => std::unreachable!(),
            } }
        })
        .collect();

    quote! {
        fn get_goto(state: usize, nonterminal: usize) -> usize {
            match state {
                #(#get_goto_branches),*
                _ => std::unreachable!(),
            }
        }
    }
}

#[inline]
fn tokenize_action<'g>(action: &LR1Action<'g, usize, usize, (i32, TokenStream)>) -> TokenStream {
    match action {
        LR1Action::Shift(dest) => quote! { shift(&mut stack, #dest, next_token); },
        LR1Action::Reduce(lhs, rhs) => {
            let action = &rhs.assoc.1;

            quote! {
                let payload = { #action }?;

                let new_top = stack.last().unwrap().0;
                let next_state = get_goto(new_top, #lhs);
                stack.push((next_state, None, Some(payload)));

                saved_input = Some(match next_token {
                    Some(next_token) => Some((next_token, next_token_n.unwrap())),
                    None => None,
                });
            }
        }
        LR1Action::Accept => quote! { break; },
    }
}
