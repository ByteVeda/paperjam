use crate::text::TextLine;

/// The type of a layout region on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionKind {
    /// Page header (top of page, running title, page number).
    Header,
    /// Page footer (bottom of page, page number, footnotes).
    Footer,
    /// A body column of text.
    BodyColumn { index: usize },
    /// A full-width section that spans all columns (e.g., title, abstract).
    FullWidth,
}

/// A rectangular region on the page containing a set of text lines.
#[derive(Debug, Clone)]
pub struct LayoutRegion {
    /// What kind of region this is.
    pub kind: RegionKind,
    /// Bounding box: (x_min, y_min, x_max, y_max).
    pub bbox: (f64, f64, f64, f64),
    /// Text lines in this region, sorted in reading order (top-to-bottom).
    pub lines: Vec<TextLine>,
}

/// Complete layout analysis for a single page.
#[derive(Debug, Clone)]
pub struct PageLayout {
    /// Page dimensions.
    pub page_width: f64,
    pub page_height: f64,
    /// Number of detected body columns (1 = single column).
    pub column_count: usize,
    /// Gutter center x-coordinates. Empty for single-column. N-1 values for N columns.
    pub gutters: Vec<f64>,
    /// All detected regions, in reading order.
    pub regions: Vec<LayoutRegion>,
}

impl PageLayout {
    /// Get all text lines in reading order across all regions.
    pub fn lines_in_reading_order(&self) -> Vec<&TextLine> {
        self.regions.iter().flat_map(|r| r.lines.iter()).collect()
    }

    /// Get all text in reading order as a single string.
    pub fn text(&self) -> String {
        self.lines_in_reading_order()
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// True if this page has a multi-column layout.
    pub fn is_multi_column(&self) -> bool {
        self.column_count > 1
    }
}

/// Options for layout analysis.
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// Minimum gutter width in points to recognize a column break. Default: 20.0.
    pub min_gutter_width: f64,
    /// Maximum number of columns to detect. Default: 4.
    pub max_columns: usize,
    /// Fraction of page height from top considered for header detection. Default: 0.08.
    pub header_zone_fraction: f64,
    /// Fraction of page height from bottom considered for footer detection. Default: 0.08.
    pub footer_zone_fraction: f64,
    /// Minimum fraction of body lines a column must contain. Default: 0.1.
    pub min_column_line_fraction: f64,
    /// Whether to detect headers and footers. Default: true.
    pub detect_headers_footers: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            min_gutter_width: 20.0,
            max_columns: 4,
            header_zone_fraction: 0.08,
            footer_zone_fraction: 0.08,
            min_column_line_fraction: 0.1,
            detect_headers_footers: true,
        }
    }
}
