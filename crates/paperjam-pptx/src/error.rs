/// Errors that can occur during PPTX processing.
#[derive(Debug, thiserror::Error)]
pub enum PptxError {
    /// The outer archive header is invalid or could not be opened.
    #[error("invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Non-archive I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parsing error.
    #[error("XML parse error: {0}")]
    Xml(String),

    /// The PPTX layout (slide XMLs, rels, metadata) was malformed.
    #[error("invalid PPTX structure: {0}")]
    InvalidStructure(String),

    /// A bounded archive read hit one of the configured safety limits
    /// (per-entry size, total-byte budget, entry count, or compression
    /// ratio), or could not locate a named entry.
    #[error(transparent)]
    Archive(#[from] paperjam_model::zip_safety::ZipSafetyError),
}

impl From<quick_xml::Error> for PptxError {
    fn from(e: quick_xml::Error) -> Self {
        PptxError::Xml(e.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for PptxError {
    fn from(e: quick_xml::events::attributes::AttrError) -> Self {
        PptxError::Xml(format!("attribute error: {e}"))
    }
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, PptxError>;
