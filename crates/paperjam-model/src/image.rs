/// Information about an extracted image from a PDF page.
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u64,
    pub height: u64,
    pub color_space: Option<String>,
    pub bits_per_component: Option<u64>,
    pub filters: Vec<String>,
    pub data: Vec<u8>,
}
