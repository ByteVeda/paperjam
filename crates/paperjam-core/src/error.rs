use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PDF parsing error: {message}")]
    Parse { message: String, offset: Option<u64> },

    #[error("Invalid PDF structure: {0}")]
    Structure(String),

    #[error("Unsupported PDF feature: {0}")]
    Unsupported(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Password required to open this document")]
    PasswordRequired,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Font decoding error: {font_name}: {message}")]
    FontDecode { font_name: String, message: String },

    #[error("Page {page} out of range (document has {total} pages)")]
    PageOutOfRange { page: usize, total: usize },

    #[error("Table extraction failed: {0}")]
    TableExtraction(String),

    #[error("Object ({0}, {1}) not found")]
    ObjectNotFound(u32, u16),

    #[error("Lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
}

pub type Result<T> = std::result::Result<T, PdfError>;
