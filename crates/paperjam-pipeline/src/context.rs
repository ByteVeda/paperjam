use std::path::{Path, PathBuf};

use paperjam_model::format::DocumentFormat;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;

/// Mutable state for a single file flowing through the pipeline.
pub struct PipelineContext {
    /// The current document bytes.
    pub bytes: Vec<u8>,
    /// The current document format.
    pub format: DocumentFormat,
    /// Original input file path (if from a file).
    pub source_path: Option<PathBuf>,

    // Extracted artifacts (populated by extraction steps).
    /// Extracted plain text.
    pub text: Option<String>,
    /// Extracted tables.
    pub tables: Option<Vec<Table>>,
    /// Extracted structure blocks.
    pub structure: Option<Vec<ContentBlock>>,
    /// Extracted markdown.
    pub markdown: Option<String>,
}

impl PipelineContext {
    /// Create a new context from a file path.
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let format = DocumentFormat::detect(path);
        Ok(Self {
            bytes,
            format,
            source_path: Some(path.to_path_buf()),
            text: None,
            tables: None,
            structure: None,
            markdown: None,
        })
    }

    /// Create a new context from bytes with a known format.
    pub fn from_bytes(bytes: Vec<u8>, format: DocumentFormat) -> Self {
        Self {
            bytes,
            format,
            source_path: None,
            text: None,
            tables: None,
            structure: None,
            markdown: None,
        }
    }

    /// Get the source filename (without path).
    pub fn filename(&self) -> String {
        self.source_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("document.{}", self.format.extension()))
    }

    /// Get the source file stem (without extension).
    pub fn stem(&self) -> String {
        self.source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".to_string())
    }

    /// Get the source file extension.
    pub fn extension(&self) -> String {
        self.format.extension().to_string()
    }

    /// Resolve a save path template with placeholders.
    /// Supports: {filename}, {stem}, {ext}
    pub fn resolve_save_path(&self, template: &str) -> PathBuf {
        let resolved = template
            .replace("{filename}", &self.filename())
            .replace("{stem}", &self.stem())
            .replace("{ext}", &self.extension());
        PathBuf::from(resolved)
    }
}
