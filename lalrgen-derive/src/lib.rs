use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    self,
    parse::{Parse, ParseStream},
    Ident, Token, Type, Variant, Visibility,
};

#[proc_macro]
pub fn parser(tok: TokenStream) -> TokenStream {
    (quote!()).into()
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
    variant: Variant,
    refname: Ident,
}

struct Action {
    tokens: TokenStream2,
}

impl Parse for Parser {
    #[inline]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        input.parse::<Token![;]>();

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
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let mut body = Vec::new();
    }
}

impl Parse for BodySymbol {
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let variant = input.parse()?;

        let refname_input;
        syn::bracketed!(refname_input in input);
        let refname = refname_input.parse()?;

        Ok(Self { variant, refname })
    }
}
