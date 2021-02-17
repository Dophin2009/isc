use std::io;

#[cfg(feature = "serde-auto")]
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum AsDiagnosticFormat {
    /// Easily parseable plain text format.
    Text,
    /// Rich text format for human-readable messages.
    Rich,
    /// JSON-formatted messages for communication protocols.
    Json,
}

/// Trait implemented by errors for diagnostic messages.
pub trait AsDiagnostic<W>
where
    W: io::Write,
{
    type Error;
    fn as_diagnostic(&self, w: &mut W, format: &AsDiagnosticFormat) -> Result<(), Self::Error>;
}

pub trait AsDiagnosticJson<W>
where
    W: io::Write,
{
    type Error;
    fn as_diagnostic_json(&self, w: &mut W) -> Result<(), Self::Error>;
}

pub trait AsDiagnosticText<W>
where
    W: io::Write,
{
    type Error;
    fn as_diagnostic_text(&self, w: &mut W) -> Result<(), Self::Error>;
}

#[cfg(feature = "serde-auto")]
impl<W, S> AsDiagnosticJson<W> for S
where
    W: io::Write,
    S: Serialize,
{
    type Error = serde_json::Error;

    #[inline]
    fn as_diagnostic_json(&self, w: &mut W) -> Result<(), Self::Error> {
        serde_json::to_writer(w, self)
    }
}
