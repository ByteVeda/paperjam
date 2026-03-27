use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PDF parsing error: {message}")]
    Parse {
        message: String,
        offset: Option<u64>,
    },

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

    #[error("Optimization failed: {0}")]
    Optimization(String),

    #[error("Annotation error: {0}")]
    Annotation(String),

    #[error("Watermark error: {0}")]
    Watermark(String),

    #[error("Sanitization error: {0}")]
    Sanitize(String),

    #[error("Redaction error: {0}")]
    Redact(String),

    #[error("Form error: {0}")]
    Form(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Signature error: {0}")]
    Signature(String),

    #[error("Object ({0}, {1}) not found")]
    ObjectNotFound(u32, u16),

    #[error("Lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
}

pub type Result<T> = std::result::Result<T, PdfError>;
