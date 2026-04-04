use serde::{Deserialize, Serialize};

/// A single step in a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    /// Extract plain text from the document.
    ExtractText {
        #[serde(default)]
        pages: Option<String>,
    },
    /// Extract tables from the document.
    ExtractTables {
        #[serde(default)]
        strategy: Option<String>,
    },
    /// Extract document structure (headings, paragraphs, lists).
    ExtractStructure,
    /// Convert the document to a different format.
    Convert { format: String },
    /// Convert the document to Markdown.
    ToMarkdown,
    /// Redact text matching a pattern (PDF only).
    Redact {
        pattern: String,
        #[serde(default)]
        case_sensitive: Option<bool>,
    },
    /// Add a text watermark (PDF only).
    Watermark {
        text: String,
        #[serde(default)]
        font_size: Option<f64>,
        #[serde(default)]
        opacity: Option<f64>,
        #[serde(default)]
        rotation: Option<f64>,
    },
    /// Optimize the document for smaller file size (PDF only).
    Optimize {
        #[serde(default)]
        strip_metadata: Option<bool>,
    },
    /// Remove potentially dangerous content (PDF only).
    Sanitize {
        #[serde(default)]
        remove_javascript: Option<bool>,
        #[serde(default)]
        remove_embedded_files: Option<bool>,
    },
    /// Encrypt the document with a password (PDF only).
    Encrypt {
        user_password: String,
        #[serde(default)]
        owner_password: Option<String>,
        #[serde(default)]
        algorithm: Option<String>,
    },
    /// Save the document to a file path.
    /// Supports placeholders: {filename}, {stem}, {ext}
    Save { path: String },
}

impl Step {
    /// Human-readable name of this step type.
    pub fn name(&self) -> &str {
        match self {
            Step::ExtractText { .. } => "extract_text",
            Step::ExtractTables { .. } => "extract_tables",
            Step::ExtractStructure => "extract_structure",
            Step::Convert { .. } => "convert",
            Step::ToMarkdown => "to_markdown",
            Step::Redact { .. } => "redact",
            Step::Watermark { .. } => "watermark",
            Step::Optimize { .. } => "optimize",
            Step::Sanitize { .. } => "sanitize",
            Step::Encrypt { .. } => "encrypt",
            Step::Save { .. } => "save",
        }
    }
}
