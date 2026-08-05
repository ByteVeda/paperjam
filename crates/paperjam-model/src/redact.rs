/// A rectangular region to redact on a specific page.
#[derive(Debug, Clone)]
pub struct RedactRegion {
    /// 1-indexed page number.
    pub page: u32,
    /// Bounding rectangle in PDF coordinates: [x1, y1, x2, y2].
    pub rect: [f64; 4],
}

/// Options for redaction.
#[derive(Debug, Clone)]
pub struct RedactOptions {
    /// Regions to redact.
    pub regions: Vec<RedactRegion>,
    /// Optional fill color [r, g, b] (0.0-1.0) for overlay rectangles.
    /// If `None`, no overlay is drawn (text is just removed).
    pub fill_color: Option<[f64; 3]>,
}

/// A single item that was redacted.
#[derive(Debug, Clone)]
pub struct RedactedItem {
    /// Page number where the item was found.
    pub page: u32,
    /// The decoded text that was removed.
    pub text: String,
    /// Bounding box of the removed text: [x1, y1, x2, y2].
    pub rect: [f64; 4],
}

/// Result statistics from a redaction operation.
#[derive(Debug, Clone)]
pub struct RedactResult {
    /// Number of pages that were modified.
    pub pages_modified: u32,
    /// Total number of text items redacted.
    pub items_redacted: u32,
    /// Detailed list of redacted items.
    pub items: Vec<RedactedItem>,
}
