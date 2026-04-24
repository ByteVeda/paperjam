pub mod document;
pub mod error;
pub mod markdown;
pub mod metadata;
pub mod parser;
mod safe_read;
pub mod structure;
pub mod table;
pub mod text;

pub use document::PptxDocument;
pub use error::PptxError;
