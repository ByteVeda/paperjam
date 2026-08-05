use base64::Engine;
use scraper::Selector;

use paperjam_model::image::ImageInfo;

use crate::document::HtmlDocument;
use crate::error::HtmlError;

/// Extract image information from `<img>` elements in a parsed HTML DOM.
pub fn extract_images_from_html(dom: &scraper::Html) -> Vec<ImageInfo> {
    let img_sel = match Selector::parse("img") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut images = Vec::new();

    for img in dom.select(&img_sel) {
        let src = img.value().attr("src").unwrap_or("");
        let width: u64 = img
            .value()
            .attr("width")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let height: u64 = img
            .value()
            .attr("height")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if let Some((color_space, data)) = decode_data_uri(src) {
            images.push(ImageInfo {
                width,
                height,
                color_space: Some(color_space),
                bits_per_component: Some(8),
                filters: Vec::new(),
                data,
            });
        } else if !src.is_empty() {
            // External image — record reference but no data.
            images.push(ImageInfo {
                width,
                height,
                color_space: None,
                bits_per_component: Some(8),
                filters: vec![src.to_string()],
                data: Vec::new(),
            });
        }
    }

    images
}

/// Try to decode a `data:` URI, returning (color_space_hint, raw_bytes).
fn decode_data_uri(src: &str) -> Option<(String, Vec<u8>)> {
    let rest = src.strip_prefix("data:")?;
    let (mime_part, encoded) = rest.split_once(";base64,")?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .ok()?;

    let color_space =
        if mime_part.contains("png") || mime_part.contains("jpeg") || mime_part.contains("jpg") {
            "RGB".to_string()
        } else {
            mime_part.to_string()
        };

    Some((color_space, data))
}

impl HtmlDocument {
    pub fn extract_images(&self) -> Result<Vec<ImageInfo>, HtmlError> {
        Ok(extract_images_from_html(&self.dom))
    }
}
