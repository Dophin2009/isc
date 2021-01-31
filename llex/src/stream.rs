use std::iter::Peekable;

use regexp2::{
    automata::{Match, DFA},
    class::CharClass,
};

pub type LexerDFA = DFA<CharClass>;

pub trait LexerDFAMatcher<T>: Clone {
    fn tokenize<I: Iterator<Item = char>>(
        &self,
        input: &mut Peekable<I>,
    ) -> Option<(T, Match<char>)>;
}

#[derive(Debug, Clone)]
pub struct LexerItem<T> {
    pub token: T,
    pub m: Match<char>,
}

impl<T> LexerItem<T> {
    #[inline]
    pub fn new(token: T, m: Match<char>) -> Self {
        Self { token, m }
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
    #[inline]
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
    I: Iterator<Item = char> + std::fmt::Debug,
{
    type Item = LexerItem<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.input.peek()?;

        let token_op = self.matcher.tokenize(&mut self.input);
        match token_op {
            // If a token was returned, return the token and the remaining input.
            Some((t, m)) => Some(LexerItem::new(t, m)),
            // If no token was returned, one input symbol should be consumed and the process
            // restarted.
            None => self.next(),
        }
    }
}
