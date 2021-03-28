use std::fmt;

#[cfg(feature = "serde-impl")]
use serde::{Deserialize, Serialize};

macro_rules! define_reserved {
    ($($variant:ident => $str:literal),*) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
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
            fn new() -> Self;
            fn variant() -> Reserved;
        }

        $(
            #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
            #[cfg_attr(feature = "serde-impl", derive(Serialize, Deserialize))]
            pub struct $variant;

            impl ReservedVariant for $variant {
                #[inline]
                fn new() -> Self {
                    Self
                }

                #[inline]
                fn variant() -> Reserved {
                    Reserved::$variant
                }
            }

            impl fmt::Display for $variant {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $str)
                }
            }
        )*

        #[cfg(feature = "diagnostic-impl")]
        mod diagnostic_impl {
            use super::*;

            use diagnostic::{AsDiagnostic, AsDiagnosticFormat};
            use std::io;

            impl<W> AsDiagnostic<W> for Reserved
            where
                W: io::Write,
            {
                type Error = io::Error;

                #[inline]
                fn as_diagnostic(&self, w: &mut W, _format: &AsDiagnosticFormat) -> Result<(), Self::Error> {
                    write!(w, "{}", self)
                }
            }

            $(
                impl<W> AsDiagnostic<W> for $variant
                where
                    W: io::Write,
                {
                    type Error = io::Error;

                    #[inline]
                    fn as_diagnostic(&self, w: &mut W, _format: &AsDiagnosticFormat) -> Result<(), Self::Error> {
                        write!(w, "{}", self)
                    }
                }
            )*
        }
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
    Return => "bye",

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

    Nequ => "!=",
    Equ => "=",
    GtEqu => ">=",
    Gt => ">",
    LtEqu => ">=",
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
