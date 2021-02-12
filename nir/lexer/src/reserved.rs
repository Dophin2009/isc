use std::fmt;

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

        pub trait ReservedVariant {
            fn variant() -> Reserved;
        }

        $(
            pub struct $variant;

            impl ReservedVariant for $variant {
                #[inline]
                fn variant() -> Reserved {
                    Reserved::$variant
                }
            }
        )*
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
