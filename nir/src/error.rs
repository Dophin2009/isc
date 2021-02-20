use std::fmt;

use ast::Spannable;
use diagnostic::{AsDiagnostic, Diagnostic};
use parser::{ExpectedToken, ParseError};
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub struct CompileError {
    pub parse: Vec<ParseError>,
}

impl fmt::Display for CompileError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: actual error printing
        write!(f, "{:#?}", self)
    }
}

impl AsDiagnostic for CompileError {
    #[inline]
    fn as_diagnostic(val: &Self) -> Vec<Diagnostic> {
        const CATEGORY: &str = "parsing";
        macro_rules! diagnostic {
            ($start:expr, $end:expr, $fmtstr:expr, $($fmtarg:expr),*) => {
                Diagnostic::new(CATEGORY.to_string(), format!($fmtstr, $($fmtarg),*), $start, $end)
            };
        };

        val.parse
            .iter()
            .map(|err| match err {
                ParseError::NoMainFunction => {
                    diagnostic!(0, 0, "required main() function not defined",)
                }
                ParseError::UndeclaredVariable(ident) => {
                    let span = ident.span();
                    diagnostic!(
                        span.start,
                        span.end,
                        "used an undeclared variable '{}'",
                        ident.name_str()
                    )
                }
                ParseError::DuplicateIdent(ident) => {
                    let span = ident.span();
                    diagnostic!(
                        span.start,
                        span.end,
                        "duplicate identifier '{}' found",
                        ident.name_str()
                    )
                }
                ParseError::LexerError => {
                    // TODO: actual positioning
                    diagnostic!(0, 0, "unexpected input",)
                }
                ParseError::UnexpectedEof(expected) => {
                    // TODO: actual positioning
                    let expected = join_expected_token(expected);
                    diagnostic!(0, 0, "unexpected EOF, expected one of {}", expected)
                }
                ParseError::UnexpectedToken(found, expected) => {
                    let expected = join_expected_token(expected);
                    let span = &found.1;
                    diagnostic!(
                        span.start,
                        span.end,
                        "unexpected '{}', expected one of {}",
                        found.0,
                        expected
                    )
                }
            })
            .collect()
    }
}

fn join_expected_token(expected: &[ExpectedToken]) -> String {
    expected
        .iter()
        .map(|et| format!("'{}'", et))
        .collect::<Vec<_>>()
        .join(", ")
}
