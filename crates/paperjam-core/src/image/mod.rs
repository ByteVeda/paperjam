use std::collections::BTreeMap;

use lopdf::ObjectId;

use crate::error::{PdfError, Result};

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

/// Extract all images from a specific page.
pub fn extract_page_images(
    doc: &lopdf::Document,
    page_number: u32,
    page_map: &BTreeMap<u32, ObjectId>,
) -> Result<Vec<ImageInfo>> {
    let page_id = page_map.get(&page_number).ok_or(PdfError::PageOutOfRange {
        page: page_number as usize,
        total: page_map.len(),
    })?;

    let images = match doc.get_page_images(*page_id) {
        Ok(imgs) => imgs,
        // Some pages have non-standard structures; return empty instead of failing
        Err(_) => return Ok(Vec::new()),
    };

    Ok(images
        .into_iter()
        .map(|img| ImageInfo {
            width: img.width as u64,
            height: img.height as u64,
            color_space: img.color_space,
            bits_per_component: img.bits_per_component.map(|b| b as u64),
            filters: img.filters.unwrap_or_default(),
            data: img.content.to_vec(),
        })
        .collect())
}
