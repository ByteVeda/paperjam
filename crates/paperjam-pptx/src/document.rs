use crate::error::{PptxError, Result};
use crate::markdown::slides_to_markdown;
use crate::metadata::PptxMetadata;
use crate::parser;
use crate::structure::slides_to_content_blocks;
use crate::table::extract_all_tables;
use crate::text::{slides_to_text, slides_to_text_lines};

use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

/// Parsed data for a single slide.
#[derive(Debug, Clone)]
pub struct SlideData {
    /// 1-based slide index.
    pub index: usize,
    /// Title extracted from the title placeholder shape, if present.
    pub title: Option<String>,
    /// All text blocks found in the slide.
    pub text_blocks: Vec<TextBlock>,
    /// Tables found in the slide.
    pub tables: Vec<Table>,
    /// Speaker notes, if present.
    pub notes: Option<String>,
}

/// A block of text within a slide shape.
#[derive(Debug, Clone)]
pub struct TextBlock {
    /// The concatenated text content.
    pub text: String,
    /// Whether this came from a title placeholder.
    pub is_title: bool,
    /// Whether this text is a bulleted list item.
    pub is_bullet: bool,
    /// Indentation level (0 = top level).
    pub level: u8,
}

/// A PPTX document loaded into memory.
pub struct PptxDocument {
    pub(crate) slides: Vec<SlideData>,
    pub(crate) metadata: PptxMetadata,
    pub(crate) raw_bytes: Vec<u8>,
}

impl PptxDocument {
    /// Open a PPTX document from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        parser::parse_pptx(bytes)
    }

    /// Return a reference to the parsed slides.
    pub fn slides(&self) -> &[SlideData] {
        &self.slides
    }
}

impl DocumentTrait for PptxDocument {
    type Error = PptxError;

    fn format(&self) -> DocumentFormat {
        DocumentFormat::Pptx
    }

    fn page_count(&self) -> usize {
        self.slides.len()
    }

    fn metadata(&self) -> std::result::Result<Metadata, PptxError> {
        Ok(self.metadata.to_model_metadata(self.slides.len()))
    }

    fn extract_text(&self) -> std::result::Result<String, PptxError> {
        Ok(slides_to_text(&self.slides))
    }

    fn extract_text_lines(&self) -> std::result::Result<Vec<TextLine>, PptxError> {
        Ok(slides_to_text_lines(&self.slides))
    }

    fn extract_tables(&self) -> std::result::Result<Vec<Table>, PptxError> {
        Ok(extract_all_tables(&self.slides))
    }

    fn extract_structure(&self) -> std::result::Result<Vec<ContentBlock>, PptxError> {
        Ok(slides_to_content_blocks(&self.slides))
    }

    fn extract_images(&self) -> std::result::Result<Vec<ImageInfo>, PptxError> {
        // Image extraction from ppt/media/ is not yet implemented.
        Ok(Vec::new())
    }

    fn bookmarks(&self) -> std::result::Result<Vec<BookmarkItem>, PptxError> {
        // PPTX files don't have a bookmark/outline concept in the same way PDFs
        // do.  We return slide titles as a flat list for navigability.
        let items = self
            .slides
            .iter()
            .filter_map(|s| {
                s.title.as_ref().map(|t| BookmarkItem {
                    title: t.clone(),
                    page: s.index,
                    level: 1,
                })
            })
            .collect();
        Ok(items)
    }

    fn to_markdown(&self) -> std::result::Result<String, PptxError> {
        Ok(slides_to_markdown(&self.slides))
    }

    fn save_to_bytes(&self) -> std::result::Result<Vec<u8>, PptxError> {
        Ok(self.raw_bytes.clone())
    }
}
