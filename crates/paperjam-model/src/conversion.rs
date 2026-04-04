use crate::validation::{PdfALevel, ValidationIssue};

/// Options for PDF/A conversion.
pub struct ConversionOptions {
    pub level: PdfALevel,
    pub force: bool,
}

/// A single action taken during conversion.
#[derive(Debug, Clone)]
pub struct ConversionAction {
    pub category: String,
    pub description: String,
    pub page: Option<u32>,
}

/// Result of a PDF/A conversion operation.
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub level: PdfALevel,
    pub success: bool,
    pub actions_taken: Vec<ConversionAction>,
    pub remaining_issues: Vec<ValidationIssue>,
}
