use std::iter::Peekable;

use regexp2::{automata::DFA, class::CharClass};

pub type LexerDFA = DFA<CharClass>;

pub trait LexerDFAMatcher<T>: Clone {
    fn tokenize<I: Iterator<Item = char>>(&self, input: &mut Peekable<I>) -> Option<T>;
}

#[derive(Debug, Clone)]
pub struct LexerItem<T> {
    pub token: T,
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

#[derive(Debug)]
pub struct LexerStream<T, M, I>
where
    M: LexerDFAMatcher<T>,
    I: Iterator<Item = char>,
{
    pub input: Peekable<I>,
    matcher: M,
    current_item: Option<LexerItem<T>>,
}

impl<T, M, I> LexerStream<T, M, I>
where
    M: LexerDFAMatcher<T>,
    I: Iterator<Item = char>,
{
    pub fn new(matcher: M, input: I) -> Self {
        Self {
            matcher,
            current_item: None,
            input: input.peekable(),
        }
    }
}

impl<'a, T, M, I> Iterator for LexerStream<T, M, I>
where
    M: LexerDFAMatcher<T>,
    I: Iterator<Item = char>,
{
    type Item = LexerItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_op = self.matcher.tokenize(&mut self.input);
        match token_op {
            // If a token was returned, return the token and the remaining input.
            Some(t) => Some(t.into()),
            // If no token was returned, one input symbol should be consumed and the process
            // restarted.
            None => self.next(),
        }
    }
}
