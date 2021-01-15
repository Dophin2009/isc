#![deny(rust_2018_idioms)]

#[cfg(test)]
mod tests;

mod parser;
mod regexp;

pub use crate::regexp::*;
