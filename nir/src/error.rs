use std::fmt;
use std::io;

use diagnostic::{AsDiagnostic, AsDiagnosticFormat, AsDiagnosticJson, AsDiagnosticText};
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

impl<W> AsDiagnostic<W> for CompileError
where
    W: io::Write,
{
    type Error = DiagnosticEmitError;

    #[inline]
    fn as_diagnostic(&self, w: &mut W, format: &AsDiagnosticFormat) -> Result<(), Self::Error> {
        match format {
            AsDiagnosticFormat::Json => self.as_diagnostic_json(w)?,
            AsDiagnosticFormat::Text => self.as_diagnostic_text(w)?,
            AsDiagnosticFormat::Rich => {}
        };

        Ok(())
    }
}

impl<W> AsDiagnosticText<W> for CompileError
where
    W: io::Write,
{
    type Error = DiagnosticEmitError;

    #[inline]
    fn as_diagnostic_text(&self, w: &mut W) -> Result<(), Self::Error> {
        for parse_err in &self.parse {
            match parse_err {
                // TODO: actual locations for these two errors
                ParseError::LexerError => writeln!(w, "0:0: unexpected input")?,
                ParseError::UnexpectedEof(ref expected) => {
                    let expected = join_expected_token(expected);
                    writeln!(w, "0:0: unexpected EOF, expected one of {}", expected)?;
                }
                ParseError::UnexpectedToken(ref found, ref expected) => {
                    let expected = join_expected_token(&expected);
                    let span = &found.1;
                    writeln!(
                        w,
                        "{}:{}: unexpected token {}, expected one of {}",
                        span.start, span.end, found.0, expected
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn join_expected_token(expected: &[ExpectedToken]) -> String {
    expected
        .iter()
        .map(|et| format!("{}", et))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Error returned when diagnostics fail to be printed.
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticEmitError {
    #[error("I/O error: {}", .0)]
    Io(#[from] io::Error),
    #[error("serialization error: {}", .0)]
    Serialize(#[from] serde_json::Error),
}
