use crate::grammar::{Grammar, Nonterminal, RuleBody, SymbolType};

#[derive(Debug, Clone)]
pub struct GrammarEliminateLR<'a, T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    grammar: &'a Grammar<T, N>,
}

impl<'a, T: 'a, N: 'a> GrammarEliminateLR<'a, T, N>
where
    T: SymbolType,
    N: Nonterminal,
{
    pub fn new_grammar(grammar: &'a Grammar<T, N>) -> Grammar<T, N> {
        let g = Self::new(grammar);

        g.eliminate().unwrap()
    }

    fn new(grammar: &'a Grammar<T, N>) -> Self {
        Self { grammar }
    }

    fn eliminate(&self) -> Option<Grammar<T, N>> {
        let rules = self.grammar.rules();

        // Arrange nonterminals in some order A0, A1, A2, ..., An.
        let rules_list: Vec<_> = rules.iter().collect();

        // Construct new rules by eliminating left recursion.
        let new_rules = Self::map_rules(&rules_list).into_iter().collect();

        Grammar::new(new_rules)
    }

    fn map_rules(rules_list: &[(&N, &Vec<RuleBody<T, N>>)]) -> Vec<(N, Vec<RuleBody<T, N>>)> {
        // for ( each i from 1 to n )
        rules_list
            .iter()
            .enumerate()
            // .skip(1)
            .flat_map(|(i, (head, bodies))| {
                // for ( each j from 1 to i - 1 )
                (0..i)
                    .map(|j| {
                        let j_rules = rules_list.get(j).unwrap();
                        let aj = j_rules.0;
                        let sigmas = j_rules.1;
                        let new_bodies: Vec<_> = Self::replace_bodies(bodies, aj, sigmas)
                            .into_iter()
                            .collect();
                        ((*head).clone(), new_bodies)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    // Replace each production of form Ai -> Aj y with Ai -> s1 y | s2 y | ... | sk y
    // where Aj -> s1 | s2 | s3 | ... | sk.
    fn replace_bodies(
        bodies: &[RuleBody<T, N>],
        aj: &N,
        sigmas: &[RuleBody<T, N>],
    ) -> Vec<RuleBody<T, N>> {
        bodies
            .iter()
            .flat_map(|body| match body.first() {
                Some(x) if x == aj => {
                    let mut gamma: Vec<_> = body.clone().into_iter().skip(1).collect();
                    sigmas
                        .iter()
                        .cloned()
                        .map(|sigma| {
                            let mut new_body = sigma.0;
                            new_body.append(&mut gamma);
                            RuleBody(new_body)
                        })
                        .collect()
                }
                _ => vec![body.clone()],
            })
            .collect()
    }
}
