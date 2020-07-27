use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, token, Expr, Ident, LitStr, Token, Type, Visibility};

pub struct Lexer {
    pub vis: Visibility,
    pub name: Ident,

    pub str_ident: Ident,
    pub return_type: Type,

    pub rules: Vec<Rule>,
}

impl Parse for Lexer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let name = {
            input.parse::<Token![fn]>()?;
            input.parse()?
        };

        let str_ident = {
            let inner;
            parenthesized!(inner in input);
            let str_ident = inner.parse()?;
            if !inner.is_empty() {
                return Err(inner.error("unexpected token after token string identifier"));
            }
            str_ident
        };

        let return_type = {
            input.parse::<Token![->]>()?;
            let ty = input.parse()?;
            input.parse::<Token![;]>()?;
            ty
        };

        let rules = {
            let mut rules = Vec::new();
            while !input.is_empty() {
                let rule = input.parse()?;
                rules.push(rule);
            }
            rules
        };

        Ok(Lexer {
            vis,
            name,
            str_ident,
            return_type,
            rules,
        })
    }
}

pub struct Rule {
    pub regexp: LitStr,
    pub action: Expr,
}

impl Rule {
    fn new(regexp: LitStr, action: Expr) -> Self {
        Self { regexp, action }
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let regexp = input.parse()?;
        input.parse::<Token![=>]>()?;

        let action = input.parse()?;

        let rule = Rule::new(regexp, action);
        Ok(rule)
    }
}
