// Everything here is just absolute spaghetti.
mod parse;

use crate::parse::{Action, BodySymbol, DestructureType, Field, Parser, Production, Rule};

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Ident, Type};

#[proc_macro]
pub fn parser(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let p = syn::parse_macro_input!(tokens as Parser);
    match parser_(p) {
        Ok(res) => res.into(),
        Err(res) => res.into(),
    }
}

fn parser_(p: Parser) -> Result<TokenStream, TokenStream> {
    let Parser {
        visibility: parser_visibility,
        name: parser_name,
        terminal_type,
        mut rules,
    } = p;

    // Get starting nonterminal. The return type of the parse method is the return type of the
    // first nonterminal.
    let start_rule = rules
        .first()
        .ok_or_else(|| span_error(Span::call_site(), "no grammar rules are specified"))?
        .clone();
    let start_rule_lhs = start_rule.nonterminal.clone();
    let parser_return_type = start_rule.return_type.clone();

    let actual_start_rule = Rule {
        nonterminal: Ident::new("__SPAGHETTI__START", Span::call_site()),
        return_type: start_rule.return_type.clone(),
        productions: vec![Production {
            body: vec![BodySymbol::Symbol {
                ident: start_rule_lhs.clone(),
                refname: Some(Field {
                    mut_token: None,
                    ident: Ident::new("ast", Span::call_site()),
                }),
            }],
            action: Action {
                expr: Expr::Verbatim(quote! { Ok(ast) }),
            },
        }],
    };
    rules.insert(0, actual_start_rule);

    // Collect nonterminals into a set to check against later.
    let grammar_nonterminals: HashMap<_, _> = rules
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let reference = NonterminalReference {
                idx: i,
                return_type: rule.return_type.clone(),
            };
            (rule.nonterminal.clone(), reference)
        })
        .collect();

    // Keep track of all terminal types to assign a number for each terminal type.
    let mut terminals = HashMap::new();
    let mut terminals_count = 0usize;

    let mut production_idx = 0;

    // Process all rules and their productions to construct information about all terminals.
    let mut rule_metas = Vec::new();
    for rule in rules.into_iter() {
        let mut production_metas = Vec::new();
        for production in rule.productions.into_iter() {
            // Keep sym_pos for disambiguation when popping from stack and destructuring.
            let mut body_meta = Vec::new();
            for (sym_pos, sym) in production.body.into_iter().enumerate() {
                let sym_meta = match sym {
                    BodySymbol::Destructure { ident, ty, fields } => {
                        let refname = TerminalRefname::Destructure(ident.clone(), ty, fields);
                        let nid = match terminals.get(&ident) {
                            Some((id, _)) => *id,
                            None => {
                                terminals.insert(ident.clone(), (terminals_count, refname.clone()));
                                let nid = terminals_count;
                                terminals_count += 1;
                                nid
                            }
                        };
                        SymbolMeta::Terminal {
                            nid,
                            base: SymbolMetaShared {
                                ty: terminal_type.clone(),
                                body_pos: sym_pos,
                            },
                            refname,
                        }
                    }
                    BodySymbol::Symbol { ident, refname } => {
                        // Check if this is referencing a nonterminal or a terminal.
                        match grammar_nonterminals.get(&ident) {
                            Some(nonterminal_ref) => {
                                // This is a nonterminal.
                                SymbolMeta::Nonterminal {
                                    nid: nonterminal_ref.idx,
                                    base: SymbolMetaShared {
                                        ty: nonterminal_ref.return_type.clone(),
                                        body_pos: sym_pos,
                                    },
                                    ident,
                                    refname,
                                }
                            }
                            None => {
                                if let Some(_) = refname {
                                    return Err(span_error(
                                        ident.span(),
                                        "unrecognized nonterminal",
                                    ));
                                }

                                let refname = TerminalRefname::Ignore;
                                let nid = match terminals.get(&ident) {
                                    Some((id, _)) => *id,
                                    None => {
                                        terminals.insert(
                                            ident.clone(),
                                            (terminals_count, refname.clone()),
                                        );
                                        let nid = terminals_count;
                                        terminals_count += 1;
                                        nid
                                    }
                                };
                                SymbolMeta::Terminal {
                                    nid,
                                    base: SymbolMetaShared {
                                        ty: terminal_type.clone(),
                                        body_pos: sym_pos,
                                    },
                                    refname,
                                }
                            }
                        }
                    }
                };
                body_meta.push(sym_meta);
            }

            let production_meta = ProductionMeta {
                idx: production_idx,
                return_type: rule.return_type.clone(),
                lhs_nonterminal: rule.nonterminal.clone(),
                body: body_meta,
                reduce_action: production.action.expr,
            };
            production_metas.push(production_meta);
            production_idx += 1;
        }
        rule_metas.push(RuleMeta {
            lhs: rule.nonterminal,
            productions: production_metas,
        });
    }

    let assoc_fn_trait = quote! {
        Fn(&mut Vec<(usize, Option<#terminal_type>, Option<PayloadNonterminal>)>) -> Result<PayloadNonterminal, ()>
    };
    let rule_inserts: Vec<_> = rule_metas
        .into_iter()
        .map(|rule| {
            let lhs = grammar_nonterminals.get(&rule.lhs).unwrap().idx;

            let rhs_set: Vec<_> = rule
                .productions
                .into_iter()
                .map(|production| {
                    let reduce_code = production.reduce_code();
                    let body_symbols: Vec<_> = production
                        .body
                        .into_iter()
                        .map(|sym| {
                            match sym {
                                SymbolMeta::Terminal { nid, base, refname } => {
                                    //
                                    quote! {
                                        ::lalrgen::lalr::Symbol::Terminal(#nid)
                                    }
                                }
                                SymbolMeta::Nonterminal {
                                    nid,
                                    base,
                                    ident,
                                    refname,
                                } => {
                                    //
                                    quote! {
                                        ::lalrgen::lalr::Symbol::Nonterminal(#nid)
                                    }
                                }
                            }
                        })
                        .collect();
                    let body = quote! { vec![ #(#body_symbols),* ] };

                    let assoc_code = reduce_code.code();
                    let assoc = quote! {
                        Box::new(|stack: &mut Vec<(usize, Option<#terminal_type>, Option<PayloadNonterminal>)>| -> Result<PayloadNonterminal, ()> {
                            #assoc_code
                        }) as Box<dyn #assoc_fn_trait>
                    };

                    quote! {
                        ::lalrgen::lalr::Rhs::new(#body, #assoc)
                    }
                })
                .collect();

            quote! {
                rules.insert(#lhs, vec![ #(#rhs_set), *]);
            }
        })
        .collect();

    let grammar_construction = quote! {
        let mut rules = std::collections::BTreeMap::new();

        #(#rule_inserts)*

        ::lalrgen::lalr::Grammar::new(0, rules).unwrap()
    };

    let payload_enum_variants: Vec<_> = grammar_nonterminals
        .iter()
        .map(|(ident, reference)| (ident, reference.return_type.clone()))
        .map(|(ident, ty)| quote! { #ident(#ty) })
        .collect();
    let terminal_map_branches: Vec<_> = terminals
        .into_iter()
        .map(|(variant, (n, refname))| {
            let variant = match refname {
                TerminalRefname::Destructure(ident, ty, _) => {
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

    Ok(quote! {
        #parser_visibility struct #parser_name {
            grammar: ::lalrgen::lalr::Grammar<
                usize,
                usize,
                Box<dyn #assoc_fn_trait>
            >,
        }

        #[derive(Debug)]
        enum PayloadNonterminal {
            #(#payload_enum_variants),*
        }

        impl #parser_name {
            #parser_visibility fn new() -> Self {
                let rules: std::collections::BTreeMap<usize, ::lalrgen::lalr::Rhs<usize, usize, ()>> = {
                    std::collections::BTreeMap::new()
                };
                let grammar = { #grammar_construction };
                Self {
                    grammar,
                }
            }

            #parser_visibility fn parse<I>(&self, mut input: I) -> Result<#parser_return_type, ()>
            where
                I: Iterator<Item = #terminal_type>,
            {
                // TODO: Figure out better way than regerating table every time.
                let table = match self.grammar.lalr1_table_by_lr1(&|_, _, _| 0) {
                    Ok(t) => t,
                    // TODO: Handle error
                    Err(_) => panic!(),
                };

                // let mut stack: Vec<(usize, Option<()>, Option<()>)> = Vec::new();
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
                                Some(ref token) => match token {
                                    #(#terminal_map_branches),*,
                                    _ => std::unreachable!("unrecognized token!"),
                                }
                                None => None,
                            };
                            (next_token, next_token_n)
                        }
                    };
                    saved_input = None;

                    let state = &table.states[current_state];
                    let get_action = match next_token_n {
                        Some(n) => state.actions.get(&n),
                        None => (&state.endmarker).as_ref(),
                    };

                    match get_action {
                        Some(action) => match action {
                            ::lalrgen::lalr::lr1::LR1Action::Shift(dest_state) => {
                                // Consume the token.
                                // Shift the state onto the stack with the current token.
                                stack.push((*dest_state, next_token, None));
                            }
                            ::lalrgen::lalr::lr1::LR1Action::Reduce(lhs, rhs) => {
                                let payload = (rhs.assoc)(&mut stack)?;

                                let new_top = stack.last().unwrap().0;
                                let next_state = table.states[new_top].goto.get(lhs).unwrap();
                                stack.push((*next_state, None, Some(payload)));

                                saved_input = Some(match next_token {
                                    Some(next_token) => Some((next_token, next_token_n.unwrap())),
                                    None => None,
                                });
                            }
                            ::lalrgen::lalr::lr1::LR1Action::Accept => {
                                // Parsing is done.
                                break;
                            }
                        }
                        None => {
                            // TODO: Handle error
                            panic!()
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
    })
}

#[derive(Clone)]
struct RuleMeta {
    lhs: Ident,
    productions: Vec<ProductionMeta>,
}

#[derive(Clone)]
struct ProductionMeta {
    idx: usize,
    /// Type returned by the action function.
    return_type: Type,
    /// Nonterminal on the lhs of this production.
    lhs_nonterminal: Ident,
    /// Metadata for each symbol in the body.
    body: Vec<SymbolMeta>,
    reduce_action: Expr,
}

#[derive(Clone)]
struct ReduceCode {
    /// Code for popping off the stack and setting the correct values.
    stack_pop: TokenStream,
    /// Code for the action function declaration itself.
    fn_decl: TokenStream,
    /// Code for calling the action function.
    fn_call: TokenStream,
    /// Code for returning from the action.
    ret: TokenStream,
}

impl ReduceCode {
    fn code(&self) -> TokenStream {
        let Self {
            stack_pop,
            fn_decl,
            fn_call,
            ret,
        } = self;

        quote! {
            #fn_decl
            #stack_pop
            #fn_call
            #ret
        }
    }
}

#[derive(Clone)]
enum SymbolMeta {
    Terminal {
        nid: usize,
        base: SymbolMetaShared,
        refname: TerminalRefname,
    },
    Nonterminal {
        nid: usize,
        base: SymbolMetaShared,
        ident: Ident,
        refname: Option<Field>,
    },
}

#[derive(Clone)]
struct SymbolMetaShared {
    /// Type of the payload associated with the symbol. For terminals, it is always the same as the
    /// return type for the Parser.
    ty: Type,
    /// Position of this terminal in the body of the production.
    body_pos: usize,
}

#[derive(Clone)]
enum TerminalRefname {
    Destructure(Ident, DestructureType, Vec<Field>),
    /// This symbol is simply ignored.
    Ignore,
}

// Information about nonterminals collected in initial pass through the nonterminals.
#[derive(Clone)]
struct NonterminalReference {
    /// Assign each nonterminal a numerical value that will be used when constructing the grammar
    /// and the parse table.
    idx: usize,
    /// Return type of the payload.
    return_type: Type,
}

impl ProductionMeta {
    /// Generate the code for the associated closure, to be called on reduction on this production.
    fn reduce_code(&self) -> ReduceCode {
        let mut pop_stmts = Vec::new();
        // Parameter declarations in the action function signature.
        let mut fn_params = Vec::new();
        // Arguments to pass to the action function.
        let mut fn_args = Vec::new();
        // Destructure statements inside in the action function.
        let mut fn_destructures = Vec::new();

        for sym_meta in self.body.iter().rev() {
            let pop_stmt;
            match sym_meta {
                SymbolMeta::Terminal {
                    nid: _,
                    base,
                    refname,
                } => match refname {
                    TerminalRefname::Ignore => {
                        pop_stmt = quote! {
                            // Pop from the stack but ignore.
                            stack.pop().unwrap();
                        };
                    }
                    TerminalRefname::Destructure(ident, destructure_ty, fields) => {
                        // Use the first field as the variable for the stack popping.
                        // TODO: Handle error properly.
                        let first_field = &fields.first().unwrap().ident;
                        pop_stmt = quote! {
                            let #first_field = {
                                let popped = stack.pop().unwrap();
                                // For terminals, payload is in the second position.
                                // Type is the token type.
                                popped.1.unwrap()
                            };
                        };

                        let param_type = base.ty.clone();
                        fn_args.push(quote! { #first_field });
                        fn_params.push(quote! { #first_field: #param_type });

                        let destructure_fields_l: Vec<_> = fields
                            .iter()
                            .map(|field| (field.mut_token, &field.ident))
                            .map(|(mut_token, ident)| quote! { #mut_token #ident })
                            .collect();
                        let destructure_fields_r: Vec<_> = fields
                            .iter()
                            .map(|field| &field.ident)
                            .map(|ident| quote! { #ident })
                            .collect();
                        let destructure_var = match destructure_ty {
                            DestructureType::Struct => {
                                quote! { #ident { #(#destructure_fields_r),* } }
                            }
                            DestructureType::TupleStruct => {
                                quote! { #ident ( #(#destructure_fields_r),* ) }
                            }
                        };

                        fn_destructures.push(quote! {
                            let ( #(#destructure_fields_l),* ) = match #first_field {
                                #param_type::#destructure_var => ( #(#destructure_fields_r),* ),
                                _ => std::unreachable!(),
                            };
                        });
                    }
                },
                SymbolMeta::Nonterminal {
                    nid: _,
                    base,
                    ident,
                    refname,
                } => match refname {
                    Some(refname) => {
                        let refname_ident = &refname.ident;
                        pop_stmt = quote! {
                            let #refname_ident = {
                                let popped = stack.pop().unwrap();
                                // For nonterminals, payload is in the third position.
                                let payload = popped.2.unwrap();
                                match payload {
                                    PayloadNonterminal::#ident(x) => x,
                                    _ => std::unreachable!(),
                                }
                            };
                        };

                        let param_type = base.ty.clone();
                        fn_args.push(quote! { #refname_ident });

                        let refname_mut = refname.mut_token;
                        fn_params.push(quote! { #refname_mut #refname_ident: #param_type });
                    }
                    None => {
                        pop_stmt = quote! {
                            // Pop from the stack but ignore.
                            stack.pop().unwrap();
                        }
                    }
                },
            };
            pop_stmts.push(pop_stmt);
        }

        let stack_pop = quote! {
            #(#pop_stmts)*
        };

        let fn_name = quote::format_ident!("reduce_{}", self.idx);
        let fn_return_type = &self.return_type;
        let fn_body = self.reduce_action.clone();
        let fn_decl = quote! {
            #[inline]
            fn #fn_name( #(#fn_params),* ) -> Result<#fn_return_type, ()> {
                #(#fn_destructures)*

                #fn_body
            }
        };

        let fn_call = quote! { let result = #fn_name( #(#fn_args),* )?; };

        let lhs_ident = &self.lhs_nonterminal;
        let ret = quote! { Ok(PayloadNonterminal::#lhs_ident(result)) };

        ReduceCode {
            stack_pop,
            fn_decl,
            fn_call,
            ret,
        }
    }
}

fn span_error(span: Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}
