/// Built-in PDF fonts (no embedding required).
#[derive(Debug, Clone)]
pub enum BuiltinFont {
    Helvetica,
    HelveticaBold,
    TimesRoman,
    TimesBold,
    Courier,
    CourierBold,
}

impl BuiltinFont {
    pub fn base_name(&self) -> &[u8] {
        match self {
            Self::Helvetica => b"Helvetica",
            Self::HelveticaBold => b"Helvetica-Bold",
            Self::TimesRoman => b"Times-Roman",
            Self::TimesBold => b"Times-Bold",
            Self::Courier => b"Courier",
            Self::CourierBold => b"Courier-Bold",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "helvetica" => Self::Helvetica,
            "helvetica-bold" | "helveticabold" => Self::HelveticaBold,
            "times-roman" | "timesroman" | "times" => Self::TimesRoman,
            "times-bold" | "timesbold" => Self::TimesBold,
            "courier" => Self::Courier,
            "courier-bold" | "courierbold" => Self::CourierBold,
            _ => Self::Helvetica,
        }
    }
}

/// Watermark position on the page.
#[derive(Debug, Clone)]
pub enum WatermarkPosition {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Custom { x: f64, y: f64 },
}

impl WatermarkPosition {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Self::Center,
            "top_left" | "topleft" => Self::TopLeft,
            "top_right" | "topright" => Self::TopRight,
            "bottom_left" | "bottomleft" => Self::BottomLeft,
            "bottom_right" | "bottomright" => Self::BottomRight,
            _ => Self::Center,
        }
    }
}

/// Whether watermark goes over or under existing content.
#[derive(Debug, Clone)]
pub enum WatermarkLayer {
    Over,
    Under,
}

impl WatermarkLayer {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "under" | "underlay" => Self::Under,
            _ => Self::Over,
        }
    }
}

/// Options for adding a text watermark.
pub struct WatermarkOptions {
    pub text: String,
    pub font_size: f64,
    pub rotation: f64,
    pub opacity: f64,
    pub color: [f64; 3],
    pub font: BuiltinFont,
    pub position: WatermarkPosition,
    pub layer: WatermarkLayer,
    pub pages: Option<Vec<u32>>,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: "WATERMARK".to_string(),
            font_size: 60.0,
            rotation: 45.0,
            opacity: 0.3,
            color: [0.5, 0.5, 0.5],
            font: BuiltinFont::Helvetica,
            position: WatermarkPosition::Center,
            layer: WatermarkLayer::Over,
            pages: None,
        }
    }
}
