use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
use lalr::{Rhs, Symbol};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self,
    parse::{Parse, ParseStream},
    token::{Brace, Bracket},
    Expr, Ident, Pat, Token, Type, Visibility,
};

#[proc_macro]
pub fn parser(tok: TokenStream) -> TokenStream {
    let Parser {
        visibility,
        name,
        rules,
    } = syn::parse_macro_input!(tok as Parser);

    // Nested action functions to be called (allows early returns in action segments).
    let mut action_fns = Vec::new();

    // Set of nonterminals in the grammar.
    let nonterminals: BTreeSet<_> = rules.iter().map(|rule| rule.nonterminal).collect();
    // Create the grammar rules from the parsed rule definitions.
    let g_rules: BTreeMap<_, _> = rules
        .into_iter()
        .map(|rule| {
            // N will simply be an Ident.
            let lhs = rule.nonterminal;

            // Create the RHS's for this nonterminal.
            let rhs_set = rule.productions.into_iter().map(|prod| {
                let (body, params): (Vec<_>, Vec<_>) = prod
                    .body
                    .into_iter()
                    .map(|body_sym| {
                        let pat = body_sym.pat;
                        let sym = match pat {
                            Some(ident) => match nonterminals.get(ident) {
                                Some(n) => Symbol::Nonterminal(n),
                                None => Symbol::Terminal(variant),
                            },
                            None => Symbol::Terminal(variant),
                        };
                        (sym, body_sym.refname)
                    })
                    .unzip();
            });

            (lhs, rhs_set)
        })
        .collect();

    (quote! {
        #[derive(Debug)]
        #visibility struct #name {

        }

        impl #name {

        }
    })
    .into()
}

struct Parser {
    visibility: Visibility,
    name: Ident,
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

struct BodySymbol {
    pat: Pat,
    refname: Option<Ident>,
}

struct Action {
    expr: Expr,
}

impl Parse for Parser {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        input.parse::<Token![;]>()?;

        let rules = {
            let mut rules = Vec::new();
            while !input.is_empty() {
                let rule: Rule = input.parse()?;
                rules.push(rule);
            }
            rules
        };

        Ok(Parser {
            visibility,
            name,
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
        let variant = input.parse()?;

        let refname = if input.peek(Bracket) {
            let refname_input;
            syn::bracketed!(refname_input in input);
            Some(refname_input.parse()?)
        } else {
            None
        };

        Ok(Self { variant, refname })
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
