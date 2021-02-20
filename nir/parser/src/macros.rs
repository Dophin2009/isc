macro_rules! reserved {
    ($variant:ident) => {
        lexer::Token::Reserved(lexer::Reserved::$variant)
    };
}

macro_rules! unexpectedeof{
    ($($expected:expr),*) => {
        $crate::error::ParseError::UnexpectedEof(vec![$($expected),*])
    };
}

macro_rules! unexpectedtoken {
    ($span:expr, $token:expr, $($expected:expr),*) => {
        $crate::error::ParseError::UnexpectedToken(ast::Spanned::new($token, $span), vec![$($expected),*])
    };
}

macro_rules! ereserved {
    ($variant:ident) => {
        $crate::error::ExpectedToken::Reserved(lexer::Reserved::$variant)
    };
}

macro_rules! eliteral {
    ($variant:ident) => {
        $crate::error::ExpectedToken::Literal(lexer::Reserved::$variant)
    };
}
