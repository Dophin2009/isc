use automata::DFA;

#[derive(Debug, Clone)]
pub struct LexerItem<T> {
    token: T,
}

impl<T> LexerItem<T> {
    pub fn new(token: T) -> Self {
        Self { token }
    }
}

impl<T> From<T> for LexerItem<T> {
    fn from(token: T) -> Self {
        LexerItem::new(token)
    }
}

#[derive(Debug, Clone)]
pub struct LexerStream<'a, T, F, G>
where
    F: Fn(usize, &str) -> Option<T>,
    G: Fn() -> T,
{
    matcher: LexerDFAMatcher<T, F, G>,
    input: &'a str,
    current_item: Option<LexerItem<T>>,
}

impl<'a, T, F, G> LexerStream<'a, T, F, G>
where
    F: Fn(usize, &str) -> Option<T>,
    G: Fn() -> T,
{
    pub fn new(matcher: LexerDFAMatcher<T, F, G>, input: &'a str) -> Self {
        Self {
            matcher,
            current_item: None,
            input,
        }
    }
}

impl<'a, T, F, G> Iterator for LexerStream<'a, T, F, G>
where
    F: Fn(usize, &str) -> Option<T>,
    G: Fn() -> T,
{
    type Item = LexerItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_op = self.matcher.tokenize(self.input);
        match token_op {
            // If a token was returned, return the token and the remaining input.
            Some((t, remaining)) => {
                self.input = remaining;
                Some(t.into())
            }
            // If no token was returned, one input symbol should be consumed and the process
            // restarted.
            None => {
                let remaining: String = self.input.chars().skip(1).collect();
                if remaining.is_empty() {
                    None
                } else {
                    self.next()
                }
            }
        }
    }
}

pub type LexerDFA = DFA<regexp2::class::CharClass>;

#[derive(Debug, Clone)]
pub struct LexerDFAMatcher<T, F, G>
where
    F: Fn(usize, &str) -> Option<T>,
    G: Fn() -> T,
{
    dfa: LexerDFA,
    match_fn: F,
    error_variant: G,
}

impl<T, F, G> LexerDFAMatcher<T, F, G>
where
    F: Fn(usize, &str) -> Option<T>,
    G: Fn() -> T,
{
    pub fn new(dfa: LexerDFA, match_fn: F, error_variant: G) -> Self {
        Self {
            dfa,
            match_fn,
            error_variant,
        }
    }

    fn tokenize<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        // Step through DFA to the find the longest match.
        let (m, final_state) = match self.dfa.find(&input.chars()) {
            Some(m) => m,
            None => return Some(((self.error_variant)(), input)),
        };

        // Execute the action expression corresponding to the final state.
        let span: std::string::String = input.chars().take(m.end()).collect();
        let token_op = (self.match_fn)(final_state, &span);

        let idx = input.char_indices().nth(m.end()).unwrap().0;
        let remaining = &input[idx..];
        token_op.map(|t| (t, remaining))
    }
}
