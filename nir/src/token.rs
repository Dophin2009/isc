use std::fmt;

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

pub(crate) trait ReservedVariant {
    fn variant() -> Reserved;
}

macro_rules! define_reserved {
    ($($variant:ident => $str:literal),*) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum Reserved {
            $($variant),*
        }

        impl fmt::Display for Reserved {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Self::$variant => write!(f, $str) ),*
                }
            }
        }

        $(
            pub(crate) struct $variant;

            impl ReservedVariant for $variant {
                #[inline]
                fn variant() -> Reserved {
                    Reserved::$variant
                }
            }
        )*
    };
}

macro_rules! reserved {
    ($variant:ident) => {
        Token::Reserved(Reserved::$variant)
    };
}

define_reserved! {
    Pub => "pub",
    Using => "using",

    Struct => "struct",
    Function => "fn",

    Let => "let",

    While => "while",
    For => "foor",
    In => "in",
    Break => "break",
    Continue => "continue",

    If => "if",
    Else => "else",

    // Symbols
    LBracket => "[",
    RBracket => "]",
    LParen => "(",
    RParen => ")",
    LBrace => "{{",
    RBrace => "}}",

    Dot => ".",
    Comma => ",",
    Colon => ":",
    DoubleColon => "::",
    Semicolon => ";",
    Arrow => "->",

    Equ => "=",
    Gt => ">",
    Lt => "<",

    Plus => "+",
    Minus => "-",
    Star => "*",
    Slash => "/",
    Exclamation => "!",

    Amp => "&",
    DoubleAmp => "&&",
    Bar => "|",
    DoubleBar => "||"
}

impl fmt::Display for Token {
    #[inline]
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

impl fmt::Display for Literal {
    #[inline]
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
    #[inline]
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
