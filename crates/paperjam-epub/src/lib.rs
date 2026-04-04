mod document;
mod error;
mod image;
mod markdown;
mod metadata;
pub mod parser;
mod structure;
mod table;
mod text;
pub mod toc;
pub mod writer;

pub use document::{ChapterData, EpubDocument, TocEntry};
pub use error::EpubError;

use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

impl DocumentTrait for EpubDocument {
    type Error = EpubError;

    fn format(&self) -> DocumentFormat {
        DocumentFormat::Epub
    }

    fn page_count(&self) -> usize {
        self.chapters.len()
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
        let items = self
            .toc_entries
            .iter()
            .map(|e| {
                // Find which chapter this entry points to.
                let href_clean = e.href.split('#').next().unwrap_or(&e.href);
                let page = self
                    .chapters
                    .iter()
                    .find(|ch| {
                        let ch_clean = ch.href.split('#').next().unwrap_or(&ch.href);
                        ch_clean == href_clean || ch_clean.ends_with(href_clean)
                    })
                    .map(|ch| ch.index + 1)
                    .unwrap_or(1);

                BookmarkItem {
                    title: e.title.clone(),
                    page,
                    level: e.level,
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
