pub mod document;
pub mod page;

use paperjam_core::error::PdfError;

/// Convert a tokio JoinError into a PdfError.
fn join_err(e: tokio::task::JoinError) -> PdfError {
    PdfError::Io(std::io::Error::other(e.to_string()))
}
