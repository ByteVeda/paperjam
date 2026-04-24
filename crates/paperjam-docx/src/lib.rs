//! DOCX (Office Open XML word-processing) support for the paperjam
//! ecosystem.
//!
//! Reads and writes `.docx` files and exposes text, tables, and metadata
//! through the `DocumentTrait` implementation on `DocxDocument`. Body
//! parsing is delegated to `docx-rs`; an internal size-capped ZIP reader
//! handles the metadata parts the upstream API does not expose.

mod document;
mod error;
mod image;
mod markdown;
mod metadata;
mod structure;
mod table;
pub(crate) mod text;

pub use document::DocxDocument;
pub use error::DocxError;

use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

impl DocumentTrait for DocxDocument {
    type Error = DocxError;

    fn format(&self) -> DocumentFormat {
        DocumentFormat::Docx
    }

    fn page_count(&self) -> usize {
        // DOCX documents don't have a fixed page count without rendering.
        1
    }

    fn metadata(&self) -> Result<Metadata, Self::Error> {
        self.extract_metadata()
    }

    fn extract_text(&self) -> Result<String, Self::Error> {
        self.extract_text()
    }

    fn extract_text_lines(&self) -> Result<Vec<TextLine>, Self::Error> {
        self.extract_text_lines()
    }

    fn extract_tables(&self) -> Result<Vec<Table>, Self::Error> {
        self.extract_tables()
    }

    fn extract_structure(&self) -> Result<Vec<ContentBlock>, Self::Error> {
        self.extract_structure()
    }

    fn extract_images(&self) -> Result<Vec<ImageInfo>, Self::Error> {
        self.extract_images()
    }

    fn bookmarks(&self) -> Result<Vec<BookmarkItem>, Self::Error> {
        // DOCX bookmarks are inline markers, not a hierarchical outline.
        // Return empty for now.
        Ok(Vec::new())
    }

    fn to_markdown(&self) -> Result<String, Self::Error> {
        self.to_markdown()
    }

    fn save_to_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        let mut buf = std::io::Cursor::new(Vec::new());
        self.inner
            .clone()
            .build()
            .pack(&mut buf)
            .map_err(|e| DocxError::Io(std::io::Error::other(e)))?;
        Ok(buf.into_inner())
    }
}
