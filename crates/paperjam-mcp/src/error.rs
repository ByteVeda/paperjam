#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("PDF error: {0}")]
    Pdf(#[from] paperjam_core::error::PdfError),

    #[error("conversion error: {0}")]
    Convert(#[from] paperjam_convert::ConvertError),

    #[error("pipeline error: {0}")]
    Pipeline(#[from] paperjam_pipeline::PipelineError),

    #[error("{0}")]
    Other(String),
}
