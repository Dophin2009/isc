use std::fmt;

macro_rules! reserved {
    ($variant:ident) => {
        Token::Reserved(Reserved::$variant)
    };
}

/// Atoms parsed by the lexer and passed to the parser.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Literal(Literal),
    Type(Type),
    Reserved(Reserved),

    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Reserved {
    Pub,
    Using,

    Struct,
    Function,

    Let,

    While,
    For,
    In,
    Break,
    Continue,

    If,
    Else,

    // Symbols
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,

    Dot,
    Comma,
    Colon,
    DoubleColon,
    Semicolon,
    Arrow,

    Equ,
    Gt,
    Lt,

    Plus,
    Minus,
    Star,
    Slash,
    Exclamation,

    Amp,
    DoubleAmp,
    Bar,
    DoubleBar,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// Token for a quoted string literal.
    Str(String),

    Integer(i64),
    Float(f64),

    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Literal(literal) => write!(f, "{}", literal),
            Token::Type(ty) => write!(f, "{}", ty),
            Token::Reserved(reserved) => write!(f, "{}", reserved),
            Token::Unknown => write!(f, ""),
        }
    }
}

impl fmt::Display for Reserved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reserved::Pub => write!(f, "export"),
            Reserved::Using => write!(f, "using"),
            Reserved::Struct => write!(f, "struct"),
            Reserved::Function => write!(f, "fn"),
            Reserved::Let => write!(f, "let"),
            Reserved::While => write!(f, "while"),
            Reserved::For => write!(f, "for"),
            Reserved::In => write!(f, "in"),
            Reserved::Break => write!(f, "break"),
            Reserved::Continue => write!(f, "continue"),
            Reserved::If => write!(f, "if"),
            Reserved::Else => write!(f, "else"),
            Reserved::LBracket => write!(f, "["),
            Reserved::RBracket => write!(f, "]"),
            Reserved::LParen => write!(f, "("),
            Reserved::RParen => write!(f, ")"),
            Reserved::LBrace => write!(f, "{{"),
            Reserved::RBrace => write!(f, "}}"),
            Reserved::Dot => write!(f, "."),
            Reserved::Comma => write!(f, ","),
            Reserved::Colon => write!(f, ":"),
            Reserved::DoubleColon => write!(f, "::"),
            Reserved::Semicolon => write!(f, ";"),
            Reserved::Arrow => write!(f, "->"),
            Reserved::Equ => write!(f, "="),
            Reserved::Gt => write!(f, ">"),
            Reserved::Lt => write!(f, "<"),
            Reserved::Plus => write!(f, "+"),
            Reserved::Minus => write!(f, "-"),
            Reserved::Star => write!(f, "*"),
            Reserved::Slash => write!(f, "/"),
            Reserved::Exclamation => write!(f, "!"),
            Reserved::Amp => write!(f, "&"),
            Reserved::DoubleAmp => write!(f, "&&"),
            Reserved::Bar => write!(f, "|"),
            Reserved::DoubleBar => write!(f, "||"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Str(s) => write!(f, r#""{}""#, s),
            Literal::Integer(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::I128 => write!(f, "i128"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
        }
    }
}
