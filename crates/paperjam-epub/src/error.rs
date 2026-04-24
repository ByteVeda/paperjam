#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    /// The outer archive header is invalid or could not be opened.
    #[error("invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Non-archive I/O failure (e.g. cloning raw bytes).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parse failure.
    #[error("XML parse error: {0}")]
    Xml(String),

    /// Chapter content failed to parse as HTML.
    #[error("HTML parse error: {0}")]
    Html(#[from] paperjam_html::HtmlError),

    /// The EPUB layout (container.xml, OPF, spine) was malformed.
    #[error("invalid EPUB structure: {0}")]
    InvalidStructure(String),

    /// A bounded archive read hit one of the configured safety limits
    /// (per-entry size, total-byte budget, entry count, or compression
    /// ratio), or could not locate a named entry.
    #[error(transparent)]
    Archive(#[from] paperjam_model::zip_safety::ZipSafetyError),
}

impl From<quick_xml::Error> for EpubError {
    fn from(e: quick_xml::Error) -> Self {
        EpubError::Xml(e.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for EpubError {
    fn from(e: quick_xml::events::attributes::AttrError) -> Self {
        EpubError::Xml(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EpubError>;
