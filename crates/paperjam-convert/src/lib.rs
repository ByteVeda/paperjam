//! Cross-format document conversion.
//!
//! Orchestrates conversion between every pair of formats supported by the
//! paperjam workspace (PDF, DOCX, XLSX, PPTX, HTML, EPUB, Markdown). Each
//! format crate is an optional dependency so consumers only pay for the
//! formats they want; features named after the source and target crates
//! gate those conversions in and out.

pub mod convert;
pub mod detect;
pub mod error;
pub mod extract;
pub mod generate;
pub mod intermediate;

pub use convert::{convert, convert_bytes, ConvertReport};
pub use detect::{detect_format, detect_format_bytes};
pub use error::ConvertError;
pub use intermediate::IntermediateDoc;
