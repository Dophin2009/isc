pub use crate::{Literal, Reserved, Token, Type};

macro_rules! reserved {
    ($variant:ident) => {
        Some(Token::Reserved(Reserved::$variant))
    };
}

macro_rules! ty {
    ($variant:ident) => {
        Some(Token::Type(Type::$variant))
    };
}

macro_rules! literal {
    ($variant:expr) => {
        Some(Token::Literal($variant))
    };
}

llex::lexer! {
    pub struct Lexer;
    pub fn stream;
    (text) -> Token, Token::Unknown;

    r"\s" => None,

    "pub" => reserved!(Pub),
    "using" => reserved!(Using),
    "struct" => reserved!(Struct),
    "fn" => reserved!(Function),
    "let" => reserved!(Let),
    "while" => reserved!(While),
    "for" => reserved!(For),
    "in" => reserved!(In),
    "break" => reserved!(Break),
    "continue" => reserved!(Continue),
    "bye" => reserved!(Return),

    "bool" => ty!(Bool),
    "char" => ty!(Char),
    "i8" => ty!(I8),
    "i16" => ty!(I16),
    "i32" => ty!(I32),
    "i64" => ty!(I64),
    "u8" => ty!(U8),
    "u16" => ty!(U16),
    "u32" => ty!(U32),
    "u64" => ty!(U64),
    "f32" => ty!(F32),
    "f64" => ty!(F64),

    r"{" => reserved!(LBrace),
    r"}" => reserved!(RBrace),
    r"\[" => reserved!(LBracket),
    r"\]" => reserved!(RBracket),
    r"\(" => reserved!(LParen),
    r"\)" => reserved!(RParen),

    r"\." => reserved!(Dot),
    r"," => reserved!(Comma),
    r"::" => reserved!(DoubleColon),
    r":" => reserved!(Colon),
    r";" => reserved!(Semicolon),
    r"->" => reserved!(Arrow),

    r"!=" => reserved!(Nequ),
    r"=" => reserved!(Equ),
    r">=" => reserved!(GtEqu),
    r">" => reserved!(Gt),
    r"<=" => reserved!(GtEqu),
    r"<" => reserved!(Lt),

    r"\+" => reserved!(Plus),
    r"-" => reserved!(Minus),
    r"\*" => reserved!(Star),
    r"/" => reserved!(Slash),

    r"!" => reserved!(Exclamation),
    r"&&" => reserved!(DoubleAmp),
    r"&" => reserved!(Amp),
    r"\|" => reserved!(Bar),
    r"\|\|" => reserved!(DoubleBar),

    "true" => literal!(Literal::Boolean(true)),
    "false" => literal!(Literal::Boolean(false)),

    r"[A-Za-z_][A-Za-z0-9_]*" => Some(Token::Ident(text.to_string())),
    r#""[^"]*""# => {
        let text = text.strip_prefix("\"").unwrap();
        let text = text.strip_suffix("\"").unwrap();
        literal!(Literal::Str(text.to_string()))
    },

    r"[0-9]+" => {
        let n = text.parse().unwrap();
        literal!(Literal::Integer(n))
    }
    r"[0-9]\.[0-9]+" => {
        let f = text.parse().unwrap();
        literal!(Literal::Float(f))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut tokens = lex("struct fn pub using let for in");

        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::Struct)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::Function)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::Pub)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::Using)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::Let)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::For)));
        assert_eq!(tokens.next(), Some(Token::Reserved(Reserved::In)));
    }

    fn lex(input: &str) -> impl Iterator<Item = Token> {
        let lexer = Lexer::new();
        let tokens: Vec<_> = lexer.stream(input.chars()).map(|item| item.token).collect();
        tokens.into_iter()
    }
}
