mod parse;

use crate::parse::{Action, BodySymbol, DestructureType, Parser, Production, Rule};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use lalr::{Grammar, Rhs, Symbol};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Type};

struct ProductionMeta {
    /// Unique index assigned to this production.
    idx: usize,
    /// Type returned by the action function.
    return_type: Type,
    /// Nonterminal on the lhs of this production.
    lhs_nonterminal: Ident,
    /// Metadata for each symbol in the body.
    body: Vec<SymbolMeta>,
    /// Information about the action function executed on reduction.
    reduce_code: ReduceCode,
}

impl ProductionMeta {
    /// Generate the code for the associated closure, to be called on reduction on this production.
    fn action(&self) -> ReduceCode {
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
                SymbolMeta::Terminal { base, refname } => match refname {
                    TerminalRefname::Ignore => {
                        pop_stmt = quote! {
                            // Pop from the stack but ignore.
                            stack.pop().unwrap();
                        };
                    }
                    TerminalRefname::Destructure(ident, destructure_ty, fields) => {
                        // Use the first field as the variable for the stack popping.
                        // TODO: Handle error properly.
                        let first_field = fields.first().unwrap();
                        pop_stmt = quote! {
                            let #first_field = {
                                let popped = stack.pop().unwrap();
                                // For terminals, payload is in the second position.
                                // Type is the token type.
                                popped.1.unwrap()
                            };
                        };

                        let param_type = base.ty;
                        fn_args.push(quote! { #first_field });
                        fn_params.push(quote! { #first_field: #param_type });

                        let destructure_var = match destructure_ty {
                            DestructureType::Struct => quote! { #ident { #(#fields),* } },
                            DestructureType::TupleStruct => quote! { #ident ( #(#fields),* ) },
                        };
                        fn_destructures.push(quote! {
                            let ( #(#fields),* ) = match #first_field {
                                #param_type::#destructure_var => ( #(#fields),* ),
                                _ => std::unreachable!(),
                            };
                        });
                    }
                },
                SymbolMeta::Nonterminal {
                    base,
                    ident,
                    refname,
                } => match refname {
                    Some(refname) => {
                        pop_stmt = quote! {
                            let #refname = {
                                let popped = stack.pop().unwrap();
                                // For nonterminals, payload is in the third position.
                                let payload = popped.2.unwrap();
                                match payload {
                                    PayloadNonterminal::#ident(x) => x,
                                    _ => std::unreachable!(),
                                }
                            };
                        };

                        let param_type = base.ty;
                        fn_args.push(quote! { #refname });
                        fn_params.push(quote! { #refname });
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

        let fn_name = quote::format_ident!("action_{}", self.idx);
        let fn_return_type = &self.return_type;
        let fn_decl = quote! {
            #[inline]
            fn #fn_name( #(#fn_params),* ) -> Result<#fn_return_type, ()> {
                #(#fn_destructures)*
            }
        };

        let fn_call = quote! {#fn_name( #(#fn_args),* )?};

        ReduceCode {
            stack_pop,
            fn_call,
            fn_decl,
        }
    }
}

struct ReduceCode {
    /// Code for popping off the stack and setting the correct values.
    stack_pop: TokenStream,
    /// Code for calling the action function.
    fn_call: TokenStream,
    /// Code for the action function declaration itself.
    fn_decl: TokenStream,
}

enum SymbolMeta {
    Terminal {
        base: SymbolMetaShared,
        refname: TerminalRefname,
    },
    Nonterminal {
        base: SymbolMetaShared,
        ident: Ident,
        refname: Option<Ident>,
    },
}

struct SymbolMetaShared {
    /// Type of the payload associated with the symbol. For terminals, it is always the same as the
    /// return type for the Parser.
    ty: Type,
    /// Position of this terminal in the body of the production.
    body_pos: usize,
}

enum TerminalRefname {
    Destructure(Ident, DestructureType, Vec<Ident>),
    /// This symbol is simply ignored.
    Ignore,
}

// Information about nonterminals collected in initial pass through the nonterminals.
struct NonterminalReference {
    /// Assign each nonterminal a numerical value that will be used when constructing the grammar
    /// and the parse table.
    idx: usize,
    /// Return type of the payload.
    return_type: Type,
}

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
        rules,
    } = p;

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

    // Get starting nonterminal. The return type of the parse method is the return type of the
    // first nonterminal.
    let start_rule = rules
        .first()
        .ok_or_else(|| span_error(Span::call_site(), "no grammar rules are specified"))?;
    let parser_return_type = start_rule.return_type.clone();

    // Process all rules and their productions to construct information about all terminals.
    for rule in rules {
        let production_metas = Vec::new();
        for production in rule.productions {
            // Keep sym_pos for disambiguation when popping from stack and destructuring.
            let body_meta = Vec::new();
            for (sym_pos, sym) in production.body.into_iter().enumerate() {
                let sym_meta = match sym {
                    BodySymbol::Destructure { ident, ty, fields } => SymbolMeta::Terminal {
                        base: SymbolMetaShared {
                            ty: terminal_type,
                            body_pos: sym_pos,
                        },
                        refname: TerminalRefname::Destructure(ident, ty, fields),
                    },
                    BodySymbol::Symbol { ident, refname } => {
                        // Check if this is referencing a nonterminal or a terminal.
                        match grammar_nonterminals.get(&ident) {
                            Some(nonterminal_ref) => {
                                // This is a nonterminal.
                                SymbolMeta::Nonterminal {
                                    base: SymbolMetaShared {
                                        ty: nonterminal_ref.return_type.clone(),
                                        body_pos: sym_pos,
                                    },
                                    ident,
                                    refname,
                                }
                            }
                            None => {
                                if refname.is_some() {
                                    return Err(span_error(
                                        refname.unwrap().span(),
                                        "[refname] can only be used for nonterminals",
                                    ));
                                }

                                SymbolMeta::Terminal {
                                    base: SymbolMetaShared {
                                        ty: terminal_type.clone(),
                                        body_pos: sym_pos,
                                    },
                                    refname: TerminalRefname::Ignore,
                                }
                            }
                        }
                    }
                };
                body_meta.push(sym_meta);
            }

            let production_meta = ProductionMeta {
                return_type: rule.return_type.clone(),
                lhs_nonterminal: rule.nonterminal.clone(),
                body: body_meta,
            };
            production_metas.push(production_meta);
        }
    }

    Ok(quote! {
        #parser_visibility struct #parser_name {
            grammar: ::lalrgen::lalr::Grammar<usize, usize, ()>,
        }

        impl #parser_name {
            #parser_visibility fn new() -> Self {
                let rules: std::collections::BTreeMap<usize, ::lalrgen::lalr::Rhs<usize, usize, ()>> = {
                    std::collections::BTreeMap::new()
                };
                let grammar = ::lalrgen::lalr::Grammar::new(0, ).unwrap();
                Self {
                    grammar,
                }
            }

            #parser_visibility fn parse<I>(&self, input: I) -> Result<#parser_return_type, ()>
            where
                I: Iterator<Item = #terminal_type>,
            {
                let mut stack = Vec::new();
                let mut current_state = 0;
                stack.push((current_state));

                while true {
                    let top_state = stack.last().unwrap().0;

                }

            }
        }
    })
}

fn span_error(span: Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}
