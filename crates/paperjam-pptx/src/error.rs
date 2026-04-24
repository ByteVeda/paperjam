/// Errors that can occur during PPTX processing.
#[derive(Debug, thiserror::Error)]
pub enum PptxError {
    /// The file is not a valid ZIP archive.
    #[error("invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// I/O error while reading the archive.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parsing error.
    #[error("XML parse error: {0}")]
    Xml(String),

    /// A required entry is missing from the PPTX archive.
    #[error("missing entry in PPTX: {0}")]
    MissingEntry(String),

    /// The PPTX structure is invalid or unsupported.
    #[error("invalid PPTX structure: {0}")]
    InvalidStructure(String),

    /// A ZIP entry exceeded the per-entry decompressed byte limit.
    #[error("PPTX entry `{name}` is too large ({size} bytes, limit {limit})")]
    EntryTooLarge { name: String, size: u64, limit: u64 },
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
