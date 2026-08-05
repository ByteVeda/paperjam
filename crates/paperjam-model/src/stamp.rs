/// Whether the stamp goes over or under existing content.
#[derive(Debug, Clone)]
pub enum StampLayer {
    Over,
    Under,
}

impl StampLayer {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "under" | "underlay" => Self::Under,
            _ => Self::Over,
        }
    }
}

/// Options for page stamping/overlay.
pub struct StampOptions {
    /// 1-based page number in the stamp document to use as the stamp.
    pub source_page: u32,
    /// Target pages in the document. None = all pages.
    pub target_pages: Option<Vec<u32>>,
    /// X offset in points.
    pub x: f64,
    /// Y offset in points.
    pub y: f64,
    /// Scale factor (default 1.0).
    pub scale: f64,
    /// Opacity (0.0-1.0, default 1.0).
    pub opacity: f64,
    /// Whether stamp goes over or under existing content.
    pub layer: StampLayer,
}

impl Default for StampOptions {
    fn default() -> Self {
        Self {
            source_page: 1,
            target_pages: None,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            opacity: 1.0,
            layer: StampLayer::Over,
        }
    }
}
