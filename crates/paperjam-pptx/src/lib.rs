//! PPTX (Office Open XML presentation) support for the paperjam ecosystem.
//!
//! Parses `.pptx` archives slide-by-slide, extracts text blocks and
//! tables from slide XML, and implements `DocumentTrait` so presentations
//! participate in the shared model (slide → page).

pub mod document;
pub mod error;
pub mod markdown;
pub mod metadata;
pub mod parser;
pub mod structure;
pub mod table;
pub mod text;

pub use document::PptxDocument;
pub use error::PptxError;
