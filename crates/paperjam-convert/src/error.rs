use paperjam_model::format::DocumentFormat;

#[derive(thiserror::Error, Debug)]
pub enum ConvertError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("extraction error: {0}")]
    Extraction(String),

    #[error("generation error: {0}")]
    Generation(String),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl ConvertError {
    pub fn unsupported(format: DocumentFormat) -> Self {
        Self::UnsupportedFormat(format.display_name().to_string())
    }
}
