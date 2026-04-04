use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

use crate::error::XlsxError;
use crate::markdown::sheets_to_markdown;
use crate::metadata::extract_metadata;
use crate::reader::read_xlsx;
use crate::structure::extract_structure;
use crate::table::extract_tables;
use crate::text::{extract_text, extract_text_lines};
use crate::writer::write_xlsx;

/// Raw data for a single worksheet.
#[derive(Debug, Clone)]
pub struct SheetData {
    /// Sheet name.
    pub name: String,
    /// Row-major cell values (all stringified).
    pub rows: Vec<Vec<String>>,
}

/// An in-memory representation of an XLSX workbook.
#[derive(Debug, Clone)]
pub struct XlsxDocument {
    /// Ordered sheet names (preserved from the workbook).
    pub(crate) sheet_names: Vec<String>,
    /// Sheet data in the same order as `sheet_names`.
    pub(crate) sheets: Vec<SheetData>,
}

impl XlsxDocument {
    /// Open an XLSX file from a byte slice.
    pub fn open_bytes(bytes: &[u8]) -> Result<Self, XlsxError> {
        read_xlsx(bytes)
    }

    /// Open an XLSX file from a filesystem path.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, XlsxError> {
        let bytes = std::fs::read(path)?;
        Self::open_bytes(&bytes)
    }

    /// Return the ordered sheet names.
    pub fn sheet_names(&self) -> &[String] {
        &self.sheet_names
    }

    /// Return a reference to all sheets.
    pub fn sheets(&self) -> &[SheetData] {
        &self.sheets
    }

    /// Look up a sheet by name.
    pub fn sheet_by_name(&self, name: &str) -> Option<&SheetData> {
        self.sheets.iter().find(|s| s.name == name)
    }

    /// Look up a sheet by zero-based index.
    pub fn sheet_by_index(&self, index: usize) -> Option<&SheetData> {
        self.sheets.get(index)
    }
}

impl DocumentTrait for XlsxDocument {
    type Error = XlsxError;

    fn format(&self) -> DocumentFormat {
        DocumentFormat::Xlsx
    }

    fn page_count(&self) -> usize {
        self.sheets.len()
    }

    fn metadata(&self) -> Result<Metadata, Self::Error> {
        Ok(extract_metadata(self))
    }

    fn extract_text(&self) -> Result<String, Self::Error> {
        Ok(extract_text(self))
    }

    fn extract_text_lines(&self) -> Result<Vec<TextLine>, Self::Error> {
        Ok(extract_text_lines(self))
    }

    fn extract_tables(&self) -> Result<Vec<Table>, Self::Error> {
        Ok(extract_tables(self))
    }

    fn extract_structure(&self) -> Result<Vec<ContentBlock>, Self::Error> {
        Ok(extract_structure(self))
    }

    fn extract_images(&self) -> Result<Vec<ImageInfo>, Self::Error> {
        // XLSX images are not extracted in v1.
        Ok(Vec::new())
    }

    fn bookmarks(&self) -> Result<Vec<BookmarkItem>, Self::Error> {
        // XLSX has no bookmark concept.
        Ok(Vec::new())
    }

    fn to_markdown(&self) -> Result<String, Self::Error> {
        Ok(sheets_to_markdown(self))
    }

    fn save_to_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        write_xlsx(self)
    }
}
