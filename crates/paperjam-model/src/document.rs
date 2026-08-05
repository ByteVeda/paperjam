use crate::bookmarks::BookmarkItem;
use crate::format::DocumentFormat;
use crate::image::ImageInfo;
use crate::metadata::Metadata;
use crate::structure::ContentBlock;
use crate::table::Table;
use crate::text::TextLine;

/// Trait that every format crate's document type can implement.
///
/// Provides a format-agnostic API for extracting content from any
/// document type. Not all methods may be meaningful for every format
/// (e.g., `bookmarks()` on an XLSX file), but implementations should
/// return an empty result rather than an error in those cases.
pub trait DocumentTrait {
    type Error: std::error::Error;

    /// The document format.
    fn format(&self) -> DocumentFormat;

    /// Number of pages (or sheets, slides, etc.).
    fn page_count(&self) -> usize;

    /// Extract document metadata.
    fn metadata(&self) -> Result<Metadata, Self::Error>;

    /// Extract all text as a single string.
    fn extract_text(&self) -> Result<String, Self::Error>;

    /// Extract text as positioned lines.
    fn extract_text_lines(&self) -> Result<Vec<TextLine>, Self::Error>;

    /// Extract all tables.
    fn extract_tables(&self) -> Result<Vec<Table>, Self::Error>;

    /// Extract document structure (headings, paragraphs, lists, tables).
    fn extract_structure(&self) -> Result<Vec<ContentBlock>, Self::Error>;

    /// Extract embedded images.
    fn extract_images(&self) -> Result<Vec<ImageInfo>, Self::Error>;

    /// Extract bookmarks/outline/TOC entries.
    fn bookmarks(&self) -> Result<Vec<BookmarkItem>, Self::Error>;

    /// Convert to markdown.
    fn to_markdown(&self) -> Result<String, Self::Error>;

    /// Serialize to bytes in the document's native format.
    fn save_to_bytes(&self) -> Result<Vec<u8>, Self::Error>;
}
