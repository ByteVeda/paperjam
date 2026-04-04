use crate::table::Table;

/// A block of structured content extracted from a page.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// A heading with an inferred level (1 = largest, up to 6).
    Heading {
        text: String,
        level: u8,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// A paragraph of body text.
    Paragraph {
        text: String,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// An item in a bulleted or numbered list.
    ListItem {
        text: String,
        indent_level: u8,
        bbox: (f64, f64, f64, f64),
        page: u32,
    },
    /// A table detected on the page.
    Table { table: Table, page: u32 },
}

impl ContentBlock {
    /// Get the block type as a string identifier.
    pub fn block_type(&self) -> &str {
        match self {
            ContentBlock::Heading { .. } => "heading",
            ContentBlock::Paragraph { .. } => "paragraph",
            ContentBlock::ListItem { .. } => "list_item",
            ContentBlock::Table { .. } => "table",
        }
    }

    /// Get the text content (empty string for tables).
    pub fn text(&self) -> &str {
        match self {
            ContentBlock::Heading { text, .. }
            | ContentBlock::Paragraph { text, .. }
            | ContentBlock::ListItem { text, .. } => text,
            ContentBlock::Table { .. } => "",
        }
    }

    /// Get the bounding box.
    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            ContentBlock::Heading { bbox, .. }
            | ContentBlock::Paragraph { bbox, .. }
            | ContentBlock::ListItem { bbox, .. } => *bbox,
            ContentBlock::Table { table, .. } => table.bbox,
        }
    }

    /// Get the page number.
    pub fn page(&self) -> u32 {
        match self {
            ContentBlock::Heading { page, .. }
            | ContentBlock::Paragraph { page, .. }
            | ContentBlock::ListItem { page, .. }
            | ContentBlock::Table { page, .. } => *page,
        }
    }
}

/// Options controlling structure extraction heuristics.
pub struct StructureOptions {
    /// Minimum font size ratio vs body text to consider a heading.
    /// Default: 1.2 (20% larger).
    pub heading_size_ratio: f64,
    /// Whether to detect list items by bullet/number prefixes.
    pub detect_lists: bool,
    /// Whether to include tables from table extraction.
    pub include_tables: bool,
    /// Whether to use layout-aware reading order (column detection, header/footer).
    /// Default: false (preserves backward compatibility).
    pub layout_aware: bool,
}

impl Default for StructureOptions {
    fn default() -> Self {
        Self {
            heading_size_ratio: 1.2,
            detect_lists: true,
            include_tables: true,
            layout_aware: false,
        }
    }
}
