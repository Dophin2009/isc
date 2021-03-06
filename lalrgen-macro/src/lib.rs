// Everything here is just absolute spaghetti.
mod codegen;
mod parse;

use crate::parse::{Action, BodySymbol, DestructureType, Field, Parser, Production, Rule};

use std::collections::{BTreeMap, HashMap};

use lalr::{Grammar, Rhs, Symbol};
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

#[inline]
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
        .ok_or_else(|| span_error(Span::call_site(), "no grammar rules are specified"))?;
    let start_rule_lhs = start_rule.nonterminal.clone();
    let parser_return_type = start_rule.return_type.clone();

    // Create S' -> S start rule.
    let actual_start = actual_start_rule(&start_rule);
    rules.insert(0, actual_start);

    // Collect nonterminals into a set to check against later.
    let grammar_nonterminals = nonterminal_references(&rules);

    // Keep track of all terminal types to assign a number for each terminal type.
    let mut grammar_terminals = HashMap::new();
    let mut terminal_idx = 0usize;

    // Assign each production a unique index.
    let mut production_idx = 0;

    // Process all rules and their productions to construct information about all terminals.
    let rule_metas: Vec<_> = rules
        .into_iter()
        .map(|rule| -> Result<_, TokenStream> {
            rule_meta(
                rule,
                &terminal_type,
                &grammar_nonterminals,
                &mut grammar_terminals,
                &mut terminal_idx,
                &mut production_idx,
            )
        })
        .collect::<Result<_, _>>()?;

    // Construct grammar.
    let grammar = match grammar(rule_metas, &grammar_nonterminals) {
        Ok(g) => g,
        // TODO: Handle error
        Err(_) => panic!(),
    };

    let table = match grammar.lalr1_table_by_lr1(&|_, rhs, _| rhs.assoc.0) {
        Ok(t) => t,
        // TODO: Handle error
        Err(_) => panic!("LR1 conflict"),
    };

    let code = codegen::codegen(
        table,
        parser_visibility,
        parser_name,
        parser_return_type,
        start_rule_lhs,
        terminal_type,
        grammar_nonterminals,
        grammar_terminals,
    );
    Ok(code)
}

#[inline]
fn actual_start_rule(original_start: &Rule) -> Rule {
    Rule {
        nonterminal: Ident::new("__SPAGHETTI__START__", Span::call_site()),
        return_type: original_start.return_type.clone(),
        productions: vec![Production {
            body: vec![BodySymbol::Symbol {
                ident: original_start.nonterminal.clone(),
                refname: Some(Field {
                    mut_token: None,
                    ident: Ident::new("ast", Span::call_site()),
                }),
            }],
            action: Action {
                expr: Expr::Verbatim(quote! { Ok(ast) }),
            },
        }],
    }
}

#[inline]
fn nonterminal_references(rules: &[Rule]) -> HashMap<Ident, NonterminalReference> {
    rules
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let reference = NonterminalReference {
                idx: i,
                return_type: rule.return_type.clone(),
            };
            (rule.nonterminal.clone(), reference)
        })
        .collect()
}

#[inline]
fn symbol_meta(
    sym: BodySymbol,
    pos: usize,
    terminal_type: &Type,
    grammar_nonterminals: &HashMap<Ident, NonterminalReference>,
    grammar_terminals: &mut HashMap<Ident, (usize, TerminalRefname)>,
    terminal_idx: &mut usize,
) -> Result<SymbolMeta, TokenStream> {
    let meta = match sym {
        BodySymbol::Destructure { ident, ty, fields } => {
            let refname = TerminalRefname::Destructure(ident.clone(), ty, fields);
            let nid = match grammar_terminals.get(&ident) {
                Some((id, _)) => *id,
                None => {
                    grammar_terminals.insert(ident.clone(), (*terminal_idx, refname.clone()));
                    let nid = *terminal_idx;
                    *terminal_idx += 1;
                    nid
                }
            };
            SymbolMeta::Terminal {
                nid,
                base: SymbolMetaShared {
                    ty: terminal_type.clone(),
                    body_pos: pos,
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
                            body_pos: pos,
                        },
                        ident,
                        refname,
                    }
                }
                None => {
                    if refname.is_some() {
                        return Err(span_error(ident.span(), "unrecognized nonterminal"));
                    }

                    let refname = TerminalRefname::Ignore;
                    let nid = match grammar_terminals.get(&ident) {
                        Some((id, _)) => *id,
                        None => {
                            grammar_terminals
                                .insert(ident.clone(), (*terminal_idx, refname.clone()));
                            let nid = *terminal_idx;
                            *terminal_idx += 1;
                            nid
                        }
                    };
                    SymbolMeta::Terminal {
                        nid,
                        base: SymbolMetaShared {
                            ty: terminal_type.clone(),
                            body_pos: pos,
                        },
                        refname,
                    }
                }
            }
        }
    };
    Ok(meta)
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn production_meta(
    production: Production,
    lhs_nonterminal: &Ident,
    return_type: &Type,
    terminal_type: &Type,
    grammar_nonterminals: &HashMap<Ident, NonterminalReference>,
    grammar_terminals: &mut HashMap<Ident, (usize, TerminalRefname)>,
    terminal_idx: &mut usize,
    production_idx: &mut usize,
) -> Result<ProductionMeta, TokenStream> {
    // Keep sym_pos for disambiguation when popping from stack and destructuring.
    let body_meta = production
        .body
        .into_iter()
        .enumerate()
        .map(|(sym_pos, sym)| {
            symbol_meta(
                sym,
                sym_pos,
                &terminal_type,
                &grammar_nonterminals,
                grammar_terminals,
                terminal_idx,
            )
        })
        .collect::<Result<_, _>>()?;

    let idx = *production_idx;
    *production_idx += 1;
    Ok(ProductionMeta {
        idx,
        return_type: return_type.clone(),
        lhs_nonterminal: lhs_nonterminal.clone(),
        body: body_meta,
        reduce_action: production.action.expr,
    })
}

#[inline]
fn rule_meta(
    rule: Rule,
    terminal_type: &Type,
    grammar_nonterminals: &HashMap<Ident, NonterminalReference>,
    grammar_terminals: &mut HashMap<Ident, (usize, TerminalRefname)>,
    terminal_idx: &mut usize,
    production_idx: &mut usize,
) -> Result<RuleMeta, TokenStream> {
    let rule_nonterminal = rule.nonterminal.clone();
    let rule_return_type = rule.return_type.clone();

    let production_metas = rule
        .productions
        .into_iter()
        .map(|production| -> Result<_, TokenStream> {
            let meta = production_meta(
                production,
                &rule_nonterminal,
                &rule_return_type,
                terminal_type,
                grammar_nonterminals,
                grammar_terminals,
                terminal_idx,
                production_idx,
            )?;
            Ok(meta)
        })
        .collect::<Result<_, _>>()?;
    Ok(RuleMeta {
        lhs: rule_nonterminal,
        productions: production_metas,
    })
}

#[inline]
fn grammar(
    rule_metas: Vec<RuleMeta>,
    grammar_nonterminals: &HashMap<Ident, NonterminalReference>,
) -> Result<Grammar<usize, usize, (i32, TokenStream)>, lalr::Error> {
    let start_rule = rule_metas.first().unwrap();
    let start = grammar_nonterminals.get(&start_rule.lhs).unwrap().idx;

    let grammar_rules: BTreeMap<_, _> = rule_metas
        .into_iter()
        .map(|rule| {
            let lhs = grammar_nonterminals.get(&rule.lhs).unwrap().idx;
            let rhs_set: Vec<_> = rule
                .productions
                .into_iter()
                .enumerate()
                .map(|(priority, production)| production.grammar_rhs(priority as i32))
                .collect();

            (lhs, rhs_set)
        })
        .collect();

    Grammar::new(start, grammar_rules)
}

#[derive(Clone)]
pub(crate) struct RuleMeta {
    pub lhs: Ident,
    pub productions: Vec<ProductionMeta>,
}

#[derive(Clone)]
pub(crate) struct ProductionMeta {
    pub idx: usize,
    /// Type returned by the action function.
    pub return_type: Type,
    /// Nonterminal on the lhs of this production.
    pub lhs_nonterminal: Ident,
    /// Metadata for each symbol in the body.
    pub body: Vec<SymbolMeta>,
    pub reduce_action: Expr,
}

impl ProductionMeta {
    fn grammar_rhs(&self, relative_priority: i32) -> Rhs<usize, usize, (i32, TokenStream)> {
        let body = self.body.iter().map(|sym| sym.grammar_symbol()).collect();

        let assoc_code = self.reduce_code().code();
        let assoc = (relative_priority, assoc_code);

        Rhs::new(body, assoc)
    }
}

#[derive(Clone)]
pub(crate) struct ReduceCode {
    /// Code for popping off the stack and setting the correct values.
    pub stack_pop: TokenStream,
    /// Code for the action function declaration itself.
    pub fn_decl: TokenStream,
    /// Code for calling the action function.
    pub fn_call: TokenStream,
    /// Code for returning from the action.
    pub ret: TokenStream,
}

#[derive(Clone)]
pub(crate) enum SymbolMeta {
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

impl SymbolMeta {
    fn grammar_symbol(&self) -> Symbol<usize, usize> {
        match *self {
            SymbolMeta::Terminal { nid, .. } => Symbol::Terminal(nid),
            SymbolMeta::Nonterminal { nid, .. } => Symbol::Nonterminal(nid),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SymbolMetaShared {
    /// Type of the payload associated with the symbol. For terminals, it is always the same as the
    /// return type for the Parser.
    pub ty: Type,
    /// Position of this terminal in the body of the production.
    pub body_pos: usize,
}

#[derive(Clone)]
pub(crate) enum TerminalRefname {
    Destructure(Ident, DestructureType, Vec<Field>),
    /// This symbol is simply ignored.
    Ignore,
}

// Information about nonterminals collected in initial pass through the nonterminals.
#[derive(Clone)]
pub(crate) struct NonterminalReference {
    /// Assign each nonterminal a numerical value that will be used when constructing the grammar
    /// and the parse table.
    idx: usize,
    /// Return type of the payload.
    return_type: Type,
}

fn span_error(span: Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}
