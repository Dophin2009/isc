#![feature(iterator_fold_self)]
#![feature(proc_macro_diagnostic)]

mod macros;

use proc_macro::TokenStream;

#[proc_macro]
pub fn lexer(tok: TokenStream) -> TokenStream {
    macros::lexer(tok)
}
