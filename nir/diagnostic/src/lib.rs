use std::io;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    category: String,
    message: String,
    start: usize,
    end: usize,
}

impl Diagnostic {
    pub const fn new(category: String, message: String, start: usize, end: usize) -> Self {
        Self {
            category,
            message,
            start,
            end,
        }
    }
}

/// Trait implemented by errors for diagnostic messages.
pub trait AsDiagnostic {
    fn as_diagnostic(val: &Self) -> Vec<Diagnostic>;

    fn emit_diagnostic<W>(
        val: &Self,
        w: &mut W,
        format: &DiagnosticFormat,
    ) -> Result<(), DiagnosticEmitError>
    where
        W: io::Write,
    {
        let diagnostics = Self::as_diagnostic(val);
        for d in diagnostics {
            match format {
                DiagnosticFormat::Json => writeln!(w, "{}", serde_json::to_string(&d)?)?,
                DiagnosticFormat::Text => {
                    writeln!(w, "{}:{}: [{}] {}", d.start, d.end, d.category, d.message)?
                }
                DiagnosticFormat::Rich => {
                    writeln!(w, "{} error: {}", d.category, d.message)?;
                    writeln!(w, "at position {}:{}", d.start, d.end)?
                }
            };
            writeln!(w)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DiagnosticFormat {
    /// Easily parseable plain text format.
    Text,
    /// Rich text format for human-readable messages.
    Rich,
    /// JSON-formatted messages for communication protocols.
    Json,
}

#[derive(Debug, thiserror::Error)]
pub enum DiagnosticEmitError {
    #[error("I/O error: {}", .0)]
    Io(#[from] io::Error),
    #[error("serialization error: {}", .0)]
    Serialization(#[from] serde_json::Error),
}
