use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),

    #[error("no files matched pattern: {0}")]
    NoFilesMatched(String),

    #[error("extraction error: {0}")]
    Extraction(String),

    #[error("generation error: {0}")]
    Generation(String),

    #[error("step error in '{step}': {message}")]
    Step { step: String, message: String },

    #[error("config error: {0}")]
    Config(String),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<paperjam_convert::ConvertError> for PipelineError {
    fn from(e: paperjam_convert::ConvertError) -> Self {
        PipelineError::Extraction(e.to_string())
    }
}

impl From<paperjam_core::error::PdfError> for PipelineError {
    fn from(e: paperjam_core::error::PdfError) -> Self {
        PipelineError::Step {
            step: "pdf_operation".to_string(),
            message: e.to_string(),
        }
    }
}

/// How to handle errors during pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    /// Stop at the first error.
    #[default]
    FailFast,
    /// Skip files that error and continue.
    Skip,
    /// Collect all errors and report at the end.
    CollectErrors,
}

pub type Result<T> = std::result::Result<T, PipelineError>;
