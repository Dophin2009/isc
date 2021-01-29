use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    Expr, Ident, Token, Type, Visibility,
};

#[derive(Clone)]
pub struct Parser {
    pub visibility: Option<Visibility>,
    pub name: Ident,
    pub terminal_type: Type,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Rule {
    /// Nonterminal of the lhs.
    pub nonterminal: Ident,
    /// Return type of the associated action.
    pub return_type: Type,
    /// Productions associated with the nonterminal.
    pub productions: Vec<Production>,
}

#[derive(Clone)]
pub struct Production {
    pub body: Vec<BodySymbol>,
    pub action: Action,
}

#[derive(Clone)]
pub enum BodySymbol {
    /// Destructure is for terminals only.
    Destructure {
        ident: Ident,
        ty: DestructureType,
        fields: Vec<Field>,
    },
    /// If refname is Some, then this is for a nonterminal.
    /// Otherwise, it may be for either a terminal or a nonterminal.
    Symbol {
        ident: Ident,
        refname: Option<Field>,
    },
}

#[derive(Clone)]
pub struct Field {
    pub mut_token: Option<Token![mut]>,
    pub ident: Ident,
}

#[derive(Clone)]
pub enum DestructureType {
    Struct,
    TupleStruct,
}

impl BodySymbol {
    pub fn ident<'a>(&'a self) -> &'a Ident {
        match *self {
            BodySymbol::Destructure { ref ident, .. } => ident,
            BodySymbol::Symbol { ref ident, .. } => ident,
        }
    }
}

#[derive(Clone)]
pub struct Action {
    pub expr: Expr,
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
        let visibility = input.parse().ok();
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
        #[inline]
        fn extract_fields<'a>(input: ParseStream<'a>) -> syn::Result<Vec<Field>> {
            let fields = Punctuated::<Field, Token![,]>::parse_terminated(input)?;
            Ok(fields.into_pairs().map(|p| p.into_value()).collect())
        }

        let ident = input.parse()?;

        let sym = if input.peek(Paren) {
            // Tuple struct destructing.
            let fields_input;
            syn::parenthesized!(fields_input in input);
            Self::Destructure {
                ident,
                ty: DestructureType::TupleStruct,
                fields: extract_fields(&fields_input)?,
            }
        } else if input.peek(Brace) {
            // Struct destructing.
            let fields_input;
            syn::braced!(fields_input in input);
            Self::Destructure {
                ident,
                ty: DestructureType::Struct,
                fields: extract_fields(&fields_input)?,
            }
        } else if input.peek(Bracket) {
            // No destructuring, but optional refname.
            let refname_input;
            syn::bracketed!(refname_input in input);
            let refname = refname_input.parse()?;
            Self::Symbol {
                ident,
                refname: Some(refname),
            }
        } else {
            Self::Symbol {
                ident,
                refname: None,
            }
        };

        Ok(sym)
    }
}

impl Parse for Field {
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let mut_token = input.parse()?;
        let ident = input.parse()?;

        Ok(Self { mut_token, ident })
    }
}

impl Parse for Action {
    #[inline]
    #[must_use]
    fn parse<'a>(input: ParseStream<'a>) -> syn::Result<Self> {
        let trailing_comma = !input.peek(Brace);
        let expr = input.parse()?;

        if trailing_comma {
            if input.peek(Brace) {
                input.parse::<Option<Token![,]>>()?;
            } else {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { expr })
    }
}
