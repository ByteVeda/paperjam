#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    #[error("invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parse error: {0}")]
    Xml(String),

    #[error("HTML parse error: {0}")]
    Html(#[from] paperjam_html::HtmlError),

    #[error("missing entry in EPUB: {0}")]
    MissingEntry(String),

    #[error("invalid EPUB structure: {0}")]
    InvalidStructure(String),
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
