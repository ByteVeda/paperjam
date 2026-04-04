/// Options for automatic table-of-contents generation.
pub struct TocOptions {
    /// Maximum heading depth to include (1-6, default 6).
    pub max_depth: u8,
    /// Font size ratio for heading detection (passed to StructureOptions).
    pub heading_size_ratio: f64,
    /// Whether to use layout-aware reading order.
    pub layout_aware: bool,
    /// If true, replace existing bookmarks. If false, append.
    pub replace_existing: bool,
}

impl Default for TocOptions {
    fn default() -> Self {
        Self {
            max_depth: 6,
            heading_size_ratio: 1.2,
            layout_aware: false,
            replace_existing: true,
        }
    }
}
