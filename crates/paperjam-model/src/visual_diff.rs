use crate::diff::DiffResult;
use crate::render::RenderedImage;

/// Mode for visual diff comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualDiffMode {
    PixelDiff,
    BboxOverlay,
    Both,
}

impl VisualDiffMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bbox" | "bboxoverlay" | "bbox_overlay" => Self::BboxOverlay,
            "both" => Self::Both,
            _ => Self::PixelDiff,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::PixelDiff => "pixel_diff",
            Self::BboxOverlay => "bbox_overlay",
            Self::Both => "both",
        }
    }
}

/// Options for visual diff comparison.
pub struct VisualDiffOptions {
    pub dpi: f32,
    pub highlight_color: [u8; 4],
    pub mode: VisualDiffMode,
    pub threshold: u8,
}

impl Default for VisualDiffOptions {
    fn default() -> Self {
        Self {
            dpi: 150.0,
            highlight_color: [255, 0, 0, 128],
            mode: VisualDiffMode::PixelDiff,
            threshold: 10,
        }
    }
}

/// Visual diff result for a single page.
#[derive(Debug, Clone)]
pub struct VisualDiffPage {
    pub page: u32,
    pub image_a: RenderedImage,
    pub image_b: RenderedImage,
    pub diff_image: RenderedImage,
    pub similarity: f64,
    pub changed_pixel_count: u64,
}

/// Complete visual diff result.
#[derive(Debug, Clone)]
pub struct VisualDiffResult {
    pub pages: Vec<VisualDiffPage>,
    pub overall_similarity: f64,
    pub text_diff: DiffResult,
}
