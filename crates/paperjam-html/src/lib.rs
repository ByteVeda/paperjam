//! HTML document support for the paperjam ecosystem.
//!
//! Parses HTML bytes via `scraper`, extracts text and tables, and
//! implements `DocumentTrait` so HTML documents share the same API
//! surface as the office formats. Also used by `paperjam-epub` for
//! chapter content (EPUB spine entries are XHTML).

mod document;
mod error;
pub mod image;
mod markdown;
pub mod metadata;
pub mod structure;
pub mod table;
pub mod text;
pub mod writer;

pub use document::HtmlDocument;
pub use error::HtmlError;

// Re-export scraper for downstream crates (e.g. paperjam-epub) that need DOM access.
pub use scraper;

use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

impl DocumentTrait for HtmlDocument {
    type Error = HtmlError;

    fn format(&self) -> DocumentFormat {
        DocumentFormat::Html
    }

    fn page_count(&self) -> usize {
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
        // Return headings as bookmark entries for navigation.
        let blocks = self.extract_structure()?;
        let items = blocks
            .iter()
            .filter_map(|b| {
                if let ContentBlock::Heading { text, level, .. } = b {
                    Some(BookmarkItem {
                        title: text.clone(),
                        page: 1,
                        level: *level as usize,
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(items)
    }

    fn to_markdown(&self) -> Result<String, Self::Error> {
        self.to_markdown()
    }

    fn save_to_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.raw_bytes.clone())
    }
}
