macro_rules! reserved {
    ($variant:ident) => {
        lexer::Token::Reserved(lexer::Reserved::$variant)
    };
}
