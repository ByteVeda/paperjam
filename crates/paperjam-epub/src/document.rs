use paperjam_html::HtmlDocument;

use crate::error::EpubError;

/// Data for a single chapter parsed from the EPUB spine.
#[derive(Debug)]
pub struct ChapterData {
    /// Chapter index (0-based, in spine order).
    pub index: usize,
    /// The chapter title (from TOC, if available).
    pub title: Option<String>,
    /// The href/path within the EPUB archive.
    pub href: String,
    /// Parsed HTML DOM for this chapter.
    pub html: HtmlDocument,
}

/// Metadata parsed from the OPF package document.
#[derive(Debug, Clone, Default)]
pub struct OpfMetadata {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub rights: Option<String>,
}

/// A TOC entry parsed from NCX or nav.xhtml.
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub href: String,
    pub level: usize,
    pub children: Vec<TocEntry>,
}

/// A parsed EPUB document.
pub struct EpubDocument {
    pub(crate) chapters: Vec<ChapterData>,
    pub(crate) opf_metadata: OpfMetadata,
    pub(crate) toc_entries: Vec<TocEntry>,
    pub(crate) raw_bytes: Vec<u8>,
    /// Image data extracted from the archive (path, bytes).
    pub(crate) archive_images: Vec<(String, Vec<u8>)>,
}

impl EpubDocument {
    /// Parse an EPUB document from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EpubError> {
        crate::parser::parse_epub(bytes)
    }

    /// Get the chapters in reading order.
    pub fn chapters(&self) -> &[ChapterData] {
        &self.chapters
    }

    /// Get the TOC entries.
    pub fn toc_entries(&self) -> &[TocEntry] {
        &self.toc_entries
    }
}
