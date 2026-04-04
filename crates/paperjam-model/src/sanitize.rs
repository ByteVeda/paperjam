/// Options controlling what to sanitize from the PDF.
pub struct SanitizeOptions {
    /// Remove JavaScript (/JS, /JavaScript name tree).
    pub remove_javascript: bool,
    /// Remove embedded files (/EmbeddedFiles, /FileAttachment annotations).
    pub remove_embedded_files: bool,
    /// Remove actions (/OpenAction, /AA, /Launch, /GoToR, /SubmitForm).
    pub remove_actions: bool,
    /// Remove link annotations.
    pub remove_links: bool,
}

impl Default for SanitizeOptions {
    fn default() -> Self {
        Self {
            remove_javascript: true,
            remove_embedded_files: true,
            remove_actions: true,
            remove_links: true,
        }
    }
}

/// A single item that was found and removed during sanitization.
#[derive(Debug, Clone)]
pub struct SanitizedItem {
    /// Category: "javascript", "embedded_file", "action", "link"
    pub category: String,
    /// Description of what was found.
    pub description: String,
    /// Page number where it was found (1-indexed), if page-level.
    pub page: Option<u32>,
}

/// Result of a sanitization operation.
#[derive(Debug, Clone)]
pub struct SanitizeResult {
    pub javascript_removed: usize,
    pub embedded_files_removed: usize,
    pub actions_removed: usize,
    pub links_removed: usize,
    pub items: Vec<SanitizedItem>,
}
