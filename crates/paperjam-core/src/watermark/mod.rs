use lopdf::{dictionary, Object, ObjectId, Stream};

use crate::document::Document;
use crate::error::{PdfError, Result};

pub use paperjam_model::watermark::*;

/// Add a text watermark to pages in the document.
pub fn add_watermark(doc: &mut Document, options: &WatermarkOptions) -> Result<()> {
    let page_map = doc.inner().get_pages();
    let total_pages = page_map.len() as u32;

    let target_pages: Vec<u32> = match &options.pages {
        Some(pages) => pages.clone(),
        None => (1..=total_pages).collect(),
    };

    // Validate page numbers
    for &p in &target_pages {
        if !page_map.contains_key(&p) {
            return Err(PdfError::PageOutOfRange {
                page: p as usize,
                total: page_map.len(),
            });
        }
    }

    // Collect page IDs and dimensions first
    let page_info: Vec<(u32, ObjectId, f64, f64)> = target_pages
        .iter()
        .filter_map(|&p| {
            let id = *page_map.get(&p)?;
            let (width, height) = get_page_dimensions(doc.inner(), id).ok()?;
            Some((p, id, width, height))
        })
        .collect();

    let inner = doc.inner_mut();

    for (_page_num, page_id, width, height) in page_info {
        apply_watermark_to_page(inner, page_id, width, height, options)?;
    }

    Ok(())
}

/// Apply watermark to a single page.
fn apply_watermark_to_page(
    doc: &mut lopdf::Document,
    page_id: ObjectId,
    width: f64,
    height: f64,
    options: &WatermarkOptions,
) -> Result<()> {
    const FONT_RESOURCE_NAME: &str = "WMFont1";
    const GS_RESOURCE_NAME: &str = "WMgs1";
    let font_resource_name = FONT_RESOURCE_NAME.as_bytes();
    let gs_resource_name = GS_RESOURCE_NAME.as_bytes();

    // 1. Create ExtGState for opacity
    let gs_dict = dictionary! {
        "Type" => "ExtGState",
        "CA" => Object::Real(options.opacity as f32),
        "ca" => Object::Real(options.opacity as f32),
    };
    let gs_id = doc.new_object_id();
    doc.objects.insert(gs_id, Object::Dictionary(gs_dict));

    // 2. Create font dictionary (built-in, no embedding)
    let font_dict = dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => Object::Name(options.font.base_name().to_vec()),
    };
    let font_id = doc.new_object_id();
    doc.objects.insert(font_id, Object::Dictionary(font_dict));

    // 3. Calculate position
    let angle_rad = options.rotation * std::f64::consts::PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Approximate text width (built-in fonts average ~0.5 * font_size per char)
    let approx_text_width = options.text.len() as f64 * options.font_size * 0.5;

    let (tx, ty) = calculate_position(
        &options.position,
        width,
        height,
        approx_text_width,
        options.font_size,
        cos_a,
        sin_a,
    );

    // 4. Build content stream
    let escaped_text = escape_pdf_string(&options.text);
    let content = format!(
        "q\n/{} gs\nBT\n/{} {} Tf\n{} {} {} rg\n{} {} {} {} {} {} Tm\n({}) Tj\nET\nQ\n",
        String::from_utf8_lossy(gs_resource_name),
        String::from_utf8_lossy(font_resource_name),
        options.font_size,
        options.color[0],
        options.color[1],
        options.color[2],
        cos_a,
        sin_a,
        -sin_a,
        cos_a,
        tx,
        ty,
        escaped_text,
    );

    let content_stream = Stream::new(dictionary! {}, content.into_bytes());
    let content_id = doc.new_object_id();
    doc.objects
        .insert(content_id, Object::Stream(content_stream));

    // 5. Update page /Contents
    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

    match page_dict.get(b"Contents") {
        Ok(Object::Reference(existing_ref)) => {
            let existing_ref = *existing_ref;
            match options.layer {
                WatermarkLayer::Over => {
                    page_dict.set(
                        "Contents",
                        Object::Array(vec![
                            Object::Reference(existing_ref),
                            Object::Reference(content_id),
                        ]),
                    );
                }
                WatermarkLayer::Under => {
                    page_dict.set(
                        "Contents",
                        Object::Array(vec![
                            Object::Reference(content_id),
                            Object::Reference(existing_ref),
                        ]),
                    );
                }
            }
        }
        Ok(Object::Array(arr)) => {
            let mut new_arr = arr.clone();
            match options.layer {
                WatermarkLayer::Over => new_arr.push(Object::Reference(content_id)),
                WatermarkLayer::Under => new_arr.insert(0, Object::Reference(content_id)),
            }
            page_dict.set("Contents", Object::Array(new_arr));
        }
        _ => {
            page_dict.set("Contents", Object::Reference(content_id));
        }
    }

    // 6. Add font and ExtGState to page /Resources
    ensure_page_resources(doc, page_id)?;

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

    let resources = page_dict
        .get_mut(b"Resources")
        .map_err(|e| PdfError::Watermark(format!("No resources: {}", e)))?;
    let resources_dict = resources
        .as_dict_mut()
        .map_err(|e| PdfError::Watermark(format!("Resources not a dict: {}", e)))?;

    // Add font
    match resources_dict.get_mut(b"Font") {
        Ok(Object::Dictionary(font_resources)) => {
            font_resources.set(FONT_RESOURCE_NAME, Object::Reference(font_id));
        }
        _ => {
            let mut font_resources = lopdf::Dictionary::new();
            font_resources.set(FONT_RESOURCE_NAME, Object::Reference(font_id));
            resources_dict.set("Font", Object::Dictionary(font_resources));
        }
    }

    // Add ExtGState
    match resources_dict.get_mut(b"ExtGState") {
        Ok(Object::Dictionary(gs_resources)) => {
            gs_resources.set(GS_RESOURCE_NAME, Object::Reference(gs_id));
        }
        _ => {
            let mut gs_resources = lopdf::Dictionary::new();
            gs_resources.set(GS_RESOURCE_NAME, Object::Reference(gs_id));
            resources_dict.set("ExtGState", Object::Dictionary(gs_resources));
        }
    }

    Ok(())
}

/// Get page dimensions from MediaBox.
fn get_page_dimensions(doc: &lopdf::Document, page_id: ObjectId) -> Result<(f64, f64)> {
    let page_obj = doc
        .get_object(page_id)
        .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

    // Try MediaBox on page, then walk up to parent
    let media_box = find_media_box(doc, page_dict)?;
    let width =
        obj_to_f64(&media_box[2]).unwrap_or(612.0) - obj_to_f64(&media_box[0]).unwrap_or(0.0);
    let height =
        obj_to_f64(&media_box[3]).unwrap_or(792.0) - obj_to_f64(&media_box[1]).unwrap_or(0.0);

    Ok((width, height))
}

/// Find MediaBox, walking up to parent if needed.
fn find_media_box<'a>(
    doc: &'a lopdf::Document,
    dict: &'a lopdf::Dictionary,
) -> Result<Vec<Object>> {
    if let Ok(Object::Array(arr)) = dict.get(b"MediaBox") {
        return Ok(arr.clone());
    }

    // Walk up to parent
    if let Ok(parent_ref) = dict.get(b"Parent") {
        if let Ok(parent_id) = parent_ref.as_reference() {
            if let Ok(parent_obj) = doc.get_object(parent_id) {
                if let Ok(parent_dict) = parent_obj.as_dict() {
                    return find_media_box(doc, parent_dict);
                }
            }
        }
    }

    // Default to US Letter
    Ok(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Real(612.0),
        Object::Real(792.0),
    ])
}

/// Ensure the page has its own /Resources dict (not inherited).
fn ensure_page_resources(doc: &mut lopdf::Document, page_id: ObjectId) -> Result<()> {
    let page_obj = doc
        .get_object(page_id)
        .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

    let has_resources = page_dict.get(b"Resources").is_ok();
    if has_resources {
        // Check if it's a reference and resolve it to an inline dict
        if let Ok(Object::Reference(res_id)) = page_dict.get(b"Resources") {
            let res_id = *res_id;
            if let Ok(res_obj) = doc.get_object(res_id) {
                let cloned = res_obj.clone();
                let page_obj = doc
                    .get_object_mut(page_id)
                    .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
                let page_dict = page_obj
                    .as_dict_mut()
                    .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;
                page_dict.set("Resources", cloned);
            }
        }
        return Ok(());
    }

    // Try to inherit from parent
    let parent_resources = {
        let page_obj = doc
            .get_object(page_id)
            .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
        let page_dict = page_obj
            .as_dict()
            .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

        if let Ok(parent_ref) = page_dict.get(b"Parent") {
            if let Ok(parent_id) = parent_ref.as_reference() {
                if let Ok(parent_obj) = doc.get_object(parent_id) {
                    if let Ok(parent_dict) = parent_obj.as_dict() {
                        if let Ok(res) = parent_dict.get(b"Resources") {
                            let resolved = match res {
                                Object::Reference(rid) => doc.get_object(*rid).ok().cloned(),
                                other => Some(other.clone()),
                            };
                            resolved
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Watermark(format!("Cannot get page: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Watermark(format!("Page not a dict: {}", e)))?;

    match parent_resources {
        Some(res) => page_dict.set("Resources", res),
        None => page_dict.set("Resources", Object::Dictionary(lopdf::Dictionary::new())),
    }

    Ok(())
}

/// Calculate watermark position based on position enum.
fn calculate_position(
    position: &WatermarkPosition,
    width: f64,
    height: f64,
    text_width: f64,
    font_size: f64,
    cos_a: f64,
    sin_a: f64,
) -> (f64, f64) {
    let margin = 50.0;

    match position {
        WatermarkPosition::Center => {
            let tx = width / 2.0 - (text_width * cos_a) / 2.0;
            let ty = height / 2.0 - (text_width * sin_a) / 2.0;
            (tx, ty)
        }
        WatermarkPosition::TopLeft => (margin, height - margin - font_size),
        WatermarkPosition::TopRight => (width - margin - text_width, height - margin - font_size),
        WatermarkPosition::BottomLeft => (margin, margin),
        WatermarkPosition::BottomRight => (width - margin - text_width, margin),
        WatermarkPosition::Custom { x, y } => (*x, *y),
    }
}

/// Escape special characters in PDF string.
fn escape_pdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}
