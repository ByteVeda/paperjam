use std::fmt;

/// Errors that can occur during XLSX processing.
#[derive(Debug)]
pub enum XlsxError {
    /// Error from the calamine XLSX reader.
    Read(calamine::XlsxError),
    /// Error from the rust_xlsxwriter writer.
    Write(rust_xlsxwriter::XlsxError),
    /// I/O error (file access, etc.).
    Io(std::io::Error),
    /// Sheet not found by name or index.
    SheetNotFound(String),
}

impl fmt::Display for XlsxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XlsxError::Read(e) => write!(f, "XLSX read error: {e}"),
            XlsxError::Write(e) => write!(f, "XLSX write error: {e}"),
            XlsxError::Io(e) => write!(f, "I/O error: {e}"),
            XlsxError::SheetNotFound(name) => write!(f, "sheet not found: {name}"),
        }
    }
}

impl std::error::Error for XlsxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            XlsxError::Read(e) => Some(e),
            XlsxError::Write(e) => Some(e),
            XlsxError::Io(e) => Some(e),
            XlsxError::SheetNotFound(_) => None,
        }
    }
}

impl From<calamine::XlsxError> for XlsxError {
    fn from(e: calamine::XlsxError) -> Self {
        XlsxError::Read(e)
    }
}

impl From<rust_xlsxwriter::XlsxError> for XlsxError {
    fn from(e: rust_xlsxwriter::XlsxError) -> Self {
        XlsxError::Write(e)
    }
}

impl From<std::io::Error> for XlsxError {
    fn from(e: std::io::Error) -> Self {
        XlsxError::Io(e)
    }
}
