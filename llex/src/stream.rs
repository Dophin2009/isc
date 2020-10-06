use std::fmt::Debug;

use automata::DFA;

pub type LexerDFA = DFA<regexp2::class::CharClass>;

pub trait LexerDFAMatcher<T>: Debug + Clone {
    fn tokenize<'a>(&self, input: &'a str) -> Option<(T, &'a str)>;
}

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
pub struct LexerStream<'a, T, M>
where
    M: LexerDFAMatcher<T>,
{
    matcher: M,
    input: &'a str,
    current_item: Option<LexerItem<T>>,
}

impl<'a, T, M> LexerStream<'a, T, M>
where
    M: LexerDFAMatcher<T>,
{
    pub fn new(matcher: M, input: &'a str) -> Self {
        Self {
            matcher,
            current_item: None,
            input,
        }
    }
}

impl<'a, T, M> Iterator for LexerStream<'a, T, M>
where
    M: LexerDFAMatcher<T>,
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
