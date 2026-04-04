#[derive(thiserror::Error, Debug)]
pub enum DocxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid DOCX: {0}")]
    Invalid(String),
}
