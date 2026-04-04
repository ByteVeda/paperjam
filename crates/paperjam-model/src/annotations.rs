/// Where a link annotation points to.
#[derive(Debug, Clone)]
pub enum LinkDestination {
    /// External URI (e.g. "https://example.com").
    Uri(String),
    /// Go to a specific page within the document.
    GoTo { page: u32 },
    /// A named destination string.
    Named(String),
}

/// Type of PDF annotation.
#[derive(Debug, Clone)]
pub enum AnnotationType {
    Text,
    Link,
    FreeText,
    Highlight,
    Underline,
    StrikeOut,
    Square,
    Circle,
    Line,
    Stamp,
    Unknown(String),
}

impl AnnotationType {
    pub fn from_name(name: &[u8]) -> Self {
        match name {
            b"Text" => Self::Text,
            b"Link" => Self::Link,
            b"FreeText" => Self::FreeText,
            b"Highlight" => Self::Highlight,
            b"Underline" => Self::Underline,
            b"StrikeOut" => Self::StrikeOut,
            b"Square" => Self::Square,
            b"Circle" => Self::Circle,
            b"Line" => Self::Line,
            b"Stamp" => Self::Stamp,
            other => Self::Unknown(String::from_utf8_lossy(other).to_string()),
        }
    }

    pub fn to_name(&self) -> &[u8] {
        match self {
            Self::Text => b"Text",
            Self::Link => b"Link",
            Self::FreeText => b"FreeText",
            Self::Highlight => b"Highlight",
            Self::Underline => b"Underline",
            Self::StrikeOut => b"StrikeOut",
            Self::Square => b"Square",
            Self::Circle => b"Circle",
            Self::Line => b"Line",
            Self::Stamp => b"Stamp",
            Self::Unknown(s) => s.as_bytes(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Link => "link",
            Self::FreeText => "free_text",
            Self::Highlight => "highlight",
            Self::Underline => "underline",
            Self::StrikeOut => "strike_out",
            Self::Square => "square",
            Self::Circle => "circle",
            Self::Line => "line",
            Self::Stamp => "stamp",
            Self::Unknown(s) => s.as_str(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "text" => Self::Text,
            "link" => Self::Link,
            "free_text" => Self::FreeText,
            "highlight" => Self::Highlight,
            "underline" => Self::Underline,
            "strike_out" => Self::StrikeOut,
            "square" => Self::Square,
            "circle" => Self::Circle,
            "line" => Self::Line,
            "stamp" => Self::Stamp,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// A parsed PDF annotation.
#[derive(Debug, Clone)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub rect: [f64; 4],
    pub contents: Option<String>,
    pub author: Option<String>,
    pub color: Option<[f64; 3]>,
    pub creation_date: Option<String>,
    pub opacity: Option<f64>,
    pub url: Option<String>,
    pub destination: Option<LinkDestination>,
}

/// Options for adding a new annotation.
pub struct AddAnnotationOptions {
    pub annotation_type: AnnotationType,
    pub rect: [f64; 4],
    pub contents: Option<String>,
    pub author: Option<String>,
    pub color: Option<[f64; 3]>,
    pub opacity: Option<f64>,
    pub quad_points: Option<Vec<f64>>,
    pub url: Option<String>,
}
