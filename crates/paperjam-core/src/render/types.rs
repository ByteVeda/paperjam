/// Image format for rendered output.
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

/// Options for page rendering.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// DPI (dots per inch) for the rendered image. Default: 150.
    pub dpi: f32,
    /// Output image format. Default: PNG.
    pub format: ImageFormat,
    /// JPEG quality (1-100), only used for JPEG format. Default: 85.
    pub quality: u8,
    /// Background color as RGB (0-255). If None, uses default (white).
    pub background_color: Option<[u8; 3]>,
    /// Target pixel width. If set, overrides DPI-based sizing.
    pub scale_to_width: Option<u32>,
    /// Target pixel height. If set, overrides DPI-based sizing.
    pub scale_to_height: Option<u32>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            dpi: 150.0,
            format: ImageFormat::Png,
            quality: 85,
            background_color: None,
            scale_to_width: None,
            scale_to_height: None,
        }
    }
}

/// A rendered page image.
#[derive(Debug, Clone)]
pub struct RenderedImage {
    /// Image data bytes (encoded in the requested format).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The format of the image data.
    pub format: ImageFormat,
    /// The page number that was rendered (1-based).
    pub page: u32,
}
