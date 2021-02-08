/// Atoms parsed by the lexer and passed to the parser.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Literal(Literal),

    Keyword(Keyword),
    Type(Type),

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

    Amp,
    Bar,

    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Export,
    Using,

    Struct,
    Function,

    Let,

    For,
    In,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// Token for a quoted string literal.
    Str(String),
    Integer(i64),
    Float(f64),
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
