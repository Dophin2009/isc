use crate::token::Token;

llex::lexer! {
    pub struct Lexer;
    pub fn stream;
    (text) -> Token, Token::Error;

    r"\s"   => None,

    r"[A-Za-z_][A-Za-z0-9_]*" => Some(Token::Atom(text.to_string())),

    r"\+"   => Some(Token::Plus),
    r"-"    => Some(Token::Minus),
    r"\*"   => Some(Token::Star),
    r"/"    => Some(Token::Slash),
    r"!"    => Some(Token::Exclamation),
    r"\?"    => Some(Token::Question),
    r":"    => Some(Token::Colon),

    r"\("   => Some(Token::LParen),
    r"\)"   => Some(Token::RParen),

    r"\["   => Some(Token::LBracket),
    r"\]"   => Some(Token::RBracket),
}
