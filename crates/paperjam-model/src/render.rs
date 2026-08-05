/// Image output format for rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
}

impl ImageFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Bmp => "bmp",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jpeg" | "jpg" => Self::Jpeg,
            "bmp" => Self::Bmp,
            _ => Self::Png,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Bmp => "bmp",
        }
    }
}

/// Options for rendering PDF pages to images.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub dpi: f32,
    pub format: ImageFormat,
    pub quality: u8,
    pub background_color: Option<[u8; 3]>,
    pub scale_to_width: Option<u32>,
    pub scale_to_height: Option<u32>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            dpi: 150.0,
            format: ImageFormat::Png,
            quality: 90,
            background_color: None,
            scale_to_width: None,
            scale_to_height: None,
        }
    }
}

/// A rendered page image.
#[derive(Debug, Clone)]
pub struct RenderedImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub page: u32,
}
