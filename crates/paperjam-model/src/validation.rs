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
            Self::A1b => "PDF/A-1b",
            Self::A1a => "PDF/A-1a",
            Self::A2b => "PDF/A-2b",
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

/// A single validation issue found during compliance checking.
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

/// PDF/UA conformance level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfUaLevel {
    Ua1,
}

impl PdfUaLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ua1 => "PDF/UA-1",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(_s: &str) -> Self {
        Self::Ua1
    }
}

/// Complete PDF/UA validation report.
#[derive(Debug, Clone)]
pub struct PdfUaReport {
    pub level: PdfUaLevel,
    pub is_compliant: bool,
    pub issues: Vec<ValidationIssue>,
    pub pages_checked: usize,
    pub structure_elements_checked: usize,
}
