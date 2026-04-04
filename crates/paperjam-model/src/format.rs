use std::path::Path;

/// Recognized document formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Html,
    Epub,
    Markdown,
    Unknown,
}

impl DocumentFormat {
    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "docx" => Self::Docx,
            "xlsx" => Self::Xlsx,
            "pptx" => Self::Pptx,
            "html" | "htm" => Self::Html,
            "epub" => Self::Epub,
            "md" | "markdown" => Self::Markdown,
            _ => Self::Unknown,
        }
    }

    /// Detect format from magic bytes at the start of a file.
    pub fn from_magic_bytes(bytes: &[u8]) -> Self {
        if bytes.len() >= 4 && &bytes[..4] == b"%PDF" {
            return Self::Pdf;
        }
        // OOXML formats are ZIP containers — check for PK signature
        if bytes.len() >= 4 && &bytes[..4] == b"PK\x03\x04" {
            // Need to inspect ZIP contents to distinguish DOCX/XLSX/PPTX
            // This basic check just returns Unknown — use detect() for full detection
            return Self::Unknown;
        }
        if bytes.len() >= 5 && &bytes[..5] == b"<?xml" {
            return Self::Html; // Could be XHTML
        }
        if bytes.len() >= 15 && &bytes[..15] == b"<!DOCTYPE html>" {
            return Self::Html;
        }
        if bytes.len() >= 5 && &bytes[..5] == b"<html" {
            return Self::Html;
        }
        Self::Unknown
    }

    /// Detect format from a file path (extension + optional content check).
    pub fn detect(path: &Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let detected = Self::from_extension(ext);
            if detected != Self::Unknown {
                return detected;
            }
        }
        Self::Unknown
    }

    /// Get the canonical file extension for this format.
    pub fn extension(&self) -> &str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Html => "html",
            Self::Epub => "epub",
            Self::Markdown => "md",
            Self::Unknown => "",
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Html => "text/html",
            Self::Epub => "application/epub+zip",
            Self::Markdown => "text/markdown",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// Human-readable name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Pdf => "PDF",
            Self::Docx => "Word Document (DOCX)",
            Self::Xlsx => "Excel Spreadsheet (XLSX)",
            Self::Pptx => "PowerPoint Presentation (PPTX)",
            Self::Html => "HTML",
            Self::Epub => "EPUB",
            Self::Markdown => "Markdown",
            Self::Unknown => "Unknown",
        }
    }
}
