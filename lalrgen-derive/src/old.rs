
pub fn parser(tok: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Parser {
        visibility,
        name,
        terminal_type,
        rules,
    } = syn::parse_macro_input!(tok as Parser);

    // Nested action functions to be called (allows early returns in action segments).
    // let mut action_fns = Vec::new();

    // Starting nonterminal of the grammar.
    let starting_rule = rules.first().expect("No grammar rules specified");
    let starting_nonterminal = starting_rule.nonterminal.clone();
    let starting_return_type = starting_rule.return_type.clone();

    // Set of nonterminals in the grammar.
    let nonterminals: BTreeSet<_> = rules.iter().map(|rule| rule.nonterminal.clone()).collect();
    // Set of terminals in the grammar.
    let mut terminals = BTreeSet::new();
    // Create the grammar rules from the parsed rule definitions.
    let mut g_rules = BTreeMap::new();

    // Vector of parser state data enum nonterminal variants.
    let mut payload_nonterminal_variants = Vec::new();
    let mut payload_nonterminal_variants_set = HashSet::new();

    // Vector of parser state data enum terminal variants.
    let mut payload_terminal_variants = Vec::new();
    let mut payload_terminal_variants_set = HashSet::new();

    for rule in rules {
        // N will simply be an Ident.
        let lhs = rule.nonterminal;
        let return_type = rule.return_type;

        // Add new nonterminal variant.
        let payload_variant_name = lhs.clone();
        let payload_variant = quote! { #payload_variant_name(#return_type) };
        if payload_nonterminal_variants_set.insert(lhs.clone()) {
            payload_nonterminal_variants.push(payload_variant);
        }

        // Create the RHS's for this nonterminal.
        let mut rhs_set = Vec::new();
        for (prod_num, prod) in rule.productions.iter().enumerate() {
            // Actual grammar rhs body.
            let mut body = Vec::new();

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
                    None => {
                        terminals.insert(ident.clone());
                        (Symbol::Terminal(ident.clone()), false)
                    }
                };
                body.push(g_sym);

                match body_sym {
                    BodySymbol::TupleStruct { ident, fields } => {
                        // Destructuring can only be used for terminals.
                        if is_nonterminal {
                            return span_error(
                                ident.span(),
                                "Destructure can only be used for terminals",
                            );
                        }

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
                        let pop_stmt = if is_nonterminal {
                            quote! {
                                let #first_field = match stack.pop().unwrap().payload {
                                    Payload::Nonterminal(payload) => match payload {
                                        PayloadNonterminal::#ident(data) => data,
                                        // Not sure if this actually unreachable?
                                        _ => std::unreachable!(),
                                    }
                                    _ => std::unreachable!(),
                                };
                            }
                        } else {
                            quote! {
                                let #first_field = match stack.pop().unwrap().payload {
                                    Payload::Terminal(payload) => match payload {
                                        Payload::Terminal::#ident(data) => data,
                                        _ => std::unreachable!(),
                                    }
                                    _ => std::unreachable!(),
                                };
                            }
                        };
                        stack_pop_stmts.push(pop_stmt);

                        action_fn_args.push(quote! { #first_field });

                        // Punctuated fields for use in action function params and
                        // destructuring let.
                        let fields_punctuated = quote!(#(#fields),*);
                        let assignment = quote! { let #ident(#fields_punctuated) = #first_field; };
                        action_fn_destructures.push(assignment);

                        // If a terminal, add it to the terminal payload variant list.
                        if !is_nonterminal {
                            let payload_variant = quote! { #ident(#terminal_type) };
                            if payload_terminal_variants_set.insert(ident.clone()) {
                                payload_terminal_variants.push(payload_variant);
                            }
                        }
                    }
                    BodySymbol::Symbol { ident, refname } => {
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
            let action_fn_body = prod.action.clone();
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
                // Pop states off the stack.
                // TODO: Handle error.
                #(#stack_pop_stmts)*

                // Call action function and return result from block.
                #action_fn_name(#(#action_fn_args),*)?
            };

            let assoc = Assoc {
                action_fn: action_fn_decl,
                action_call: assoc_code,
            };
            let rhs = Rhs::new(body, assoc);
            rhs_set.push(rhs);
        }

        g_rules.insert(lhs, rhs_set);
    }

    let grammar = Grammar::new(starting_nonterminal, g_rules).unwrap();
    let table = match grammar.lalr1_table_by_lr1(&|_, _, _| 0) {
        Ok(t) => t,
        Err(_) => panic!(),
    };

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
                enum Payload {
                    Nonterminal(NonterminalPayload),
                    Terminal(Token)
                }

                enum NonterminalPayload {
                    #(#payload_nonterminal_variants),*
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
                    let next = input.next();

                    // let action = match current_state {
                        // #state => match next {
                            // Some(#token_type::#ident(#ident_fields)) => #action_variant,
                        // }
                    // }

                    // match action {
                        // LR1Action::Shift(dest) => {
                            // state_stack.push(dest)
                            // current_state = dest;
                        // }
                        // LR1Action::Reduce(prod) => match prod {
                            // 0 => {
                                // let payload = {
                                    // #code
                                // };

                                // let new_state = (#dest_state, payload);
                                // state_stack.push(new_state);
                            // }
                        // }
                        // LR1Action::Accept => break,
                        // LR1Action::Error => panic!(),
                    // }

                }

            }

        }
    })
    .into()
}
