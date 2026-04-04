use crate::structure::StructureOptions;

/// Controls how Markdown output is formatted.
pub struct MarkdownOptions {
    /// Offset added to all heading levels (e.g., 1 makes # become ##).
    /// Clamped so final level is in 1..=6. Default: 0.
    pub heading_offset: u8,
    /// Separator inserted between pages. Default: "---".
    pub page_separator: String,
    /// Whether to include page number comments. Default: false.
    pub include_page_numbers: bool,
    /// Format for page number annotations ({n} = page number). Default: "<!-- page {n} -->".
    pub page_number_format: String,
    /// Whether to use HTML tables instead of pipe tables. Default: false.
    pub html_tables: bool,
    /// Whether the first row of tables is a header row. Default: true.
    pub table_header_first_row: bool,
    /// Whether to strip original list markers and normalize to "- ". Default: true.
    pub normalize_list_markers: bool,
    /// Structure extraction options.
    pub structure_options: StructureOptions,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            heading_offset: 0,
            page_separator: "---".to_string(),
            include_page_numbers: false,
            page_number_format: "<!-- page {n} -->".to_string(),
            html_tables: false,
            table_header_first_row: true,
            normalize_list_markers: true,
            structure_options: StructureOptions::default(),
        }
    }
}
