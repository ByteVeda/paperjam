//! Tokio-native async wrappers around paperjam's blocking operations.
//!
//! Each heavy operation (`open`, `save`, `render`, `to_markdown`,
//! `merge`, `redact_text`, ...) is re-exposed as an `async fn` that runs
//! the blocking work on `tokio::spawn_blocking`. This is what powers the
//! `paperjam.aopen` / `paperjam.arender_*` / `paperjam.amerge` helpers on
//! the Python side.

pub mod document;
pub mod generic;
pub mod page;

#[cfg(feature = "convert")]
pub mod convert;

use paperjam_core::error::PdfError;

/// Convert a tokio JoinError into a PdfError.
fn join_err(e: tokio::task::JoinError) -> PdfError {
    PdfError::Io(std::io::Error::other(e.to_string()))
}
