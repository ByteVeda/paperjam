//! PDF validation: PDF/A conformance and PDF/UA accessibility checks.

mod pdf_a;
mod pdf_ua;

use lopdf::Object;

pub use pdf_a::validate_pdf_a;
pub use pdf_ua::{validate_pdf_ua, PdfUaLevel, PdfUaReport};

/// PDF/A conformance level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfALevel {
    A1b,
    A1a,
    A2b,
}

impl PdfALevel {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "1a" | "a1a" => Self::A1a,
            "2b" | "a2b" => Self::A2b,
            _ => Self::A1b,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::A1b => "1b",
            Self::A1a => "1a",
            Self::A2b => "2b",
        }
    }
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// A single validation issue found.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub rule: String,
    pub message: String,
    pub page: Option<u32>,
}

/// Complete PDF/A validation report.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub level: PdfALevel,
    pub is_compliant: bool,
    pub issues: Vec<ValidationIssue>,
    pub fonts_checked: usize,
    pub pages_checked: usize,
}

/// Get resolved resources dictionary from a page dict.
pub(crate) fn get_resources(
    doc: &lopdf::Document,
    page_dict: &lopdf::Dictionary,
) -> Option<lopdf::Dictionary> {
    match page_dict.get(b"Resources") {
        Ok(Object::Dictionary(d)) => Some(d.clone()),
        Ok(Object::Reference(id)) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .cloned(),
        _ => {
            if let Ok(parent_ref) = page_dict.get(b"Parent") {
                if let Ok(parent_id) = parent_ref.as_reference() {
                    if let Ok(parent_obj) = doc.get_object(parent_id) {
                        if let Ok(parent_dict) = parent_obj.as_dict() {
                            return get_resources(doc, parent_dict);
                        }
                    }
                }
            }
            None
        }
    }
}
