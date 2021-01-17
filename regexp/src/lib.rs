#![deny(rust_2018_idioms)]
#![deny(future_incompatible)]

#[cfg(test)]
mod tests;

mod parser;
mod regexp;

pub use crate::regexp::*;
