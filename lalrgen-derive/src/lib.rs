use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
use lalr::{Grammar, Rhs, Symbol};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma, Paren},
    Expr, FieldPat, Ident, Pat, PatPath, PatStruct, PatTuple, PatTupleStruct, Token, Type,
    Visibility,
};

#[proc_macro]
pub fn parser(tok: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Parser {
        visibility,
        name,
        terminal_type,
        rules,
    } = syn::parse_macro_input!(tok as Parser);

    // Nested action functions to be called (allows early returns in action segments).
    let mut action_fns = Vec::new();

    // Starting nonterminal of the grammar.
    let starting_rule = rules.first().unwrap();
    let starting_nonterminal = starting_rule.nonterminal;
    let starting_return_type = starting_rule.return_type;

    // Set of nonterminals in the grammar.
    let nonterminals: BTreeSet<_> = rules.iter().map(|rule| rule.nonterminal).collect();
    // Create the grammar rules from the parsed rule definitions.
    let g_rules = BTreeMap::new();

    // Vector of parser state data enum variants.
    let payload_variants = Vec::new();

    for rule in rules {
        // N will simply be an Ident.
        let lhs = rule.nonterminal;
        let return_type = rule.return_type;

        // Nonterminal variant
        let payload_variant_name = lhs;
        let payload_variant = quote! { #payload_variant_name(#return_type) };

        // Create the RHS's for this nonterminal.
        let rhs_set = Vec::new();
        for (prod_num, prod) in rule.productions.iter().enumerate() {
            // Actual grammar rhs body.
            let mut body = Vec::new();

            // Number of states to pop off the stack when reducing.
            let pop_count = prod.body.len();

            let action_fn_name =
                quote::format_ident!("{}_{}", lhs.to_string().to_lowercase(), prod_num);
            // Arguments to pass into generated action function.
            let mut action_fn_args = Vec::new();
            // Parameters for generated action function.
            let mut action_fn_params = Vec::new();
            // Destructuring `let`s for tuple struct destructures.
            let mut action_fn_destructures = Vec::new();

            // Stack pop statements to set args for calling action function.
            // Must be reversed in the order it's populated when inserting into final generated
            // code.
            let mut stack_pop_stmts = Vec::new();

            for (sym_num, body_sym) in prod.body.iter().enumerate() {
                // Check if the ident is a nonterminal or terminal and add it the body vec.
                let ident = body_sym.ident();
                let (g_sym, is_nonterminal) = match nonterminals.get(ident) {
                    Some(_) => (Symbol::Nonterminal(ident.clone()), true),
                    None => (Symbol::Terminal(ident.clone()), false),
                };
                body.push(g_sym);

                match body_sym {
                    BodySymbol::TupleStruct { ident, fields } => {
                        // First field to use as whole tuple struct's param name in action
                        // function definition.
                        let first_field = fields.first().unwrap();
                        let first_field = quote::format_ident!(
                            "{}_{}",
                            first_field.to_string().to_lowercase(),
                            sym_num
                        );

                        // Add first field as parameter (later destructured inside action
                        // function).
                        action_fn_params.push(quote! { #first_field: #ident });

                        // Add first field as arg (to pass in to action function).
                        stack_pop_stmts.push(quote! {
                            // Pop off state off stack.
                            // TODO: Handle error
                            let #first_field = match stack.pop().unwrap() {
                                Some(state) => match state.payload {
                                    StatePayload::Nonterminal(payload) => match payload {
                                        StatePayloadNonterminal::#ident(data) => data,
                                        // Not sure if this actually unreachable?
                                        _ => std::unreachable!(),
                                    }
                                    StatePayload::Terminal(payload) => match payload {
                                        StatePayloadTerminal::#ident(data) => data,
                                        _ => std::unreachable!(),
                                    }
                                }

                            };
                        });
                        action_fn_args.push(quote! { #first_field });

                        // Punctuated fields for use in action function params and
                        // destructuring let.
                        let fields_puncutated = quote!(#(#fields),*);
                        let assignment = quote! { let #ident(#fields_puncutated) = #first_field; };
                        action_fn_destructures.push(assignment);
                    }
                    BodySymbol::Symbol { ident, refname } => {
                        unimplemented!();
                        // match refname {
                        // // If there's a refname, generate the assignment, args, and params.
                        // Some(refname) => {
                        // action_fn_args = quote! { #refname };
                        // action_fn_params = quote! { #refname }
                        // } // If no refname, leave them blank.
                        // }
                    }
                };
            }

            // Construct associated action as token stream.
            let action_fn_body = prod.action;
            let action_fn_decl = quote! {
                #[inline]
                fn #action_fn_name(#(#action_fn_params),*) -> Result<#return_type, ()> {
                    #(#action_fn_destructures)*

                    #action_fn_body
                }
            };

            // Code to run when reducing on this produciton.
            stack_pop_stmts.reverse();
            let assoc_code = quote! {
                // Declare action function.
                #action_fn_decl

                // Pop states off the stack.
                // TODO: Handle error.
                #(#stack_pop_stmts)*

                // Call action function.
                let result = #action_fn_name(#(#action_fn_args),*)?;

                // Push new state to stack.
                state_stack.push()
            };

            let rhs = Rhs::new(body, assoc_code);
            rhs_set.push(rhs);
        }

        g_rules.insert(lhs, rhs_set);
    }

    let grammar = Grammar::new(starting_nonterminal, g_rules).unwrap();
    let table = grammar.lalr1_table_by_lr1(&|_, _, _| 0).unwrap();

    let initial_state = table.initial;

    (quote! {
        #[derive(Debug)]
        #visibility struct #name {
        }

        impl #name {
            // TODO: Real error variant
            #visibility fn parse<I>(&self, input: I) -> Result<#starting_return_type, ()>
            where
                I: Iterator<Item = #terminal_type>
            {
                // Define state payload enum.
                enum StatePayload {
                    #(#payload_variants),*
                }

                enum LR1Action {
                    Reduce(usize),
                    Shift(usize),
                    Accept,
                    Error,
                }

                // Parser state stack.
                let mut state_stack: Vec<(usize, StatePayload)> = Vec::new();
                // Current state.
                let mut current_state = #initial_state;

                while true {
                    // Get next input symbol.
                    let next = match input.next() {
                        Some(next) => match next {
                            Token::Ident(ident) => Some(Token),
                            // TODO: Handle error properly
                            _ => panic!("Unexpected token")
                        }
                        None => None,
                    };

                    let action = match current_state {
                        0 => match next {
                            Some(Token::Ident(ident)) => LR1Action::Shift(1),
                            _ => LR1Action::Error,
                        }
                    };

                    match action {
                        LR1Action::Shift(dest) => {
                            state_stack.push(dest)
                            current_state = dest;
                        }
                        LR1Action::Reduce(prod) => match prod {
                            0 => {
                                #code
                            }
                        }
                        LR1Action::Accept => break,
                        LR1Action::Error => panic!(),
                    }

                }

            }

        }
    })
    .into()
}

struct Parser {
    visibility: Visibility,
    name: Ident,
    terminal_type: Type,
    rules: Vec<Rule>,
}

struct Rule {
    /// Nonterminal of the lhs.
    nonterminal: Ident,
    /// Return type of the associated action.
    return_type: Type,
    /// Productions associated with the nonterminal.
    productions: Vec<Production>,
}

struct Production {
    body: Vec<BodySymbol>,
    action: Action,
}

enum BodySymbol {
    /// Tuple struct destructure
    TupleStruct { ident: Ident, fields: Vec<Ident> },
    Symbol {
        ident: Ident,
        refname: Option<Ident>,
    },
}

impl BodySymbol {
    fn ident<'a>(&'a self) -> &'a Ident {
        match *self {
            BodySymbol::TupleStruct { ref ident, .. } => ident,
            BodySymbol::Symbol { ref ident, .. } => ident,
        }
    }
}

struct Action {
    expr: Expr,
}

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.expr.to_tokens(tokens)
    }
}

impl Parse for Parser {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name = input.parse()?;

        input.parse::<Token![<]>()?;
        let terminal_type = input.parse()?;
        input.parse::<Token![>]>()?;

        input.parse::<Token![;]>()?;

        let rules = {
            let mut rules = Vec::new();
            while !input.is_empty() {
                let rule: Rule = input.parse()?;
                rules.push(rule);
            }
            rules
        };

        Ok(Self {
            visibility,
            name,
            terminal_type,
            rules,
        })
    }
}

impl Parse for Rule {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        //  nonterminal : return_type {
        //      symbol[ref] symbol[ref] .. => {
        //          ..
        //      }
        //      symbol[ref] symbol[ref] .. => {
        //          ..
        //      }
        //      => {
        //          ..
        //      }
        //  }
        let nonterminal = input.parse()?;
        input.parse::<Token![:]>()?;
        let return_type = input.parse()?;

        // Begin parsing productions.
        let rhs_input;
        syn::braced!(rhs_input in input);

        let productions = {
            let mut productions = Vec::new();
            while !rhs_input.is_empty() {
                let production = rhs_input.parse()?;
                productions.push(production);
            }

            if productions.is_empty() {
                return Err(rhs_input.error("No productions specified for this nonterminal"));
            }

            productions
        };

        Ok(Self {
            nonterminal,
            return_type,
            productions,
        })
    }
}

impl Parse for Production {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let mut body = Vec::new();
        while !input.peek(Token![=>]) {
            let symbol = input.parse()?;
            body.push(symbol);
        }

        input.parse::<Token![=>]>()?;

        let action = input.parse()?;

        Ok(Self { body, action })
    }
}

impl Parse for BodySymbol {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let ident = input.parse()?;

        let sym = if input.peek(Paren) {
            // Tuple struct destructing
            let fields_input;
            syn::parenthesized!(fields_input in input);
            let fields = Punctuated::<Ident, Token![,]>::parse_terminated(&fields_input)?;
            Self::TupleStruct {
                ident,
                fields: fields.into_pairs().map(|p| p.into_value()).collect(),
            }
        } else if input.peek(Bracket) {
            // No destructuring, but refname
            let refname_input;
            syn::bracketed!(refname_input in input);
            let refname = refname_input.parse()?;
            Self::Symbol { ident, refname }
        } else {
            Self::Symbol {
                ident,
                refname: None,
            }
        };

        Ok(sym)
    }
}

impl Parse for Action {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let trailing_comma = !input.peek(Brace);
        let expr = input.parse()?;

        if trailing_comma {
            input.parse::<Token![,]>()?;
        }

        Ok(Self { expr })
    }
}
