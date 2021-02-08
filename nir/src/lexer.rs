use crate::token::{Keyword, Literal, Token, Type};

macro_rules! token {
    ($variant:ident) => {
        Some(Token::$variant)
    };
}

macro_rules! kw {
    ($variant:ident) => {
        Some(Token::Keyword(Keyword::$variant))
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

    "export" => kw!(Export),
    "using" => kw!(Using),
    "struct" => kw!(Struct),
    "fn" => kw!(Function),
    "let" => kw!(Let),
    "while" => kw!(While),
    "for" => kw!(For),
    "in" => kw!(In),
    "break" => kw!(Break),
    "continue" => kw!(Continue),

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

    r"{" => token!(LBrace),
    r"}" => token!(RBrace),
    r"\[" => token!(LBracket),
    r"\]" => token!(RBracket),
    r"\(" => token!(LParen),
    r"\)" => token!(RParen),

    r"\." => token!(Dot),
    r"," => token!(Comma),
    r"::" => token!(DoubleColon),
    r":" => token!(Colon),
    r";" => token!(Semicolon),
    r"->" => token!(Arrow),

    r"=" => token!(Equ),
    r">" => token!(Gt),
    r"<" => token!(Lt),

    r"\+" => token!(Plus),
    r"-" => token!(Minus),
    r"\*" => token!(Star),
    r"/" => token!(Slash),

    r"&&" => token!(DoubleAmp),
    r"&" => token!(Amp),
    r"\|" => token!(Bar),
    r"\|\|" => token!(DoubleBar),
    r"!" => token!(Exclamation),

    "true" => literal!(Literal::Boolean(true)),
    "false" => literal!(Literal::Boolean(false)),

    r"[A-Za-z_][A-Za-z0-9_]*" => Some(Token::Ident(text.to_string())),
    r#"".*""# => literal!(Literal::Str(text.to_string())),

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
        let mut tokens = lex("struct fn export using let for in");

        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::Struct)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::Function)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::Export)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::Using)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::Let)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::For)));
        assert_eq!(tokens.next(), Some(Token::Keyword(Keyword::In)));
    }

    fn lex(input: &str) -> impl Iterator<Item = Token> {
        let lexer = Lexer::new();
        let tokens: Vec<_> = lexer.stream(input.chars()).map(|item| item.token).collect();
        tokens.into_iter()
    }
}
