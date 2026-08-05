use crate::error::Result;
use crate::text::font::FontInfo;

/// Load all fonts from a page's Resources dictionary.
pub fn load_page_fonts(
    doc: &lopdf::Document,
    page_dict: &lopdf::Dictionary,
) -> Result<Vec<FontInfo>> {
    let resources = match get_resources(doc, page_dict) {
        Some(res) => res,
        None => return Ok(Vec::new()),
    };

    let font_dict = match resources.get(b"Font") {
        Ok(font_ref) => {
            let (_, obj) = doc.dereference(font_ref).unwrap_or((None, font_ref));
            match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => return Ok(Vec::new()),
            }
        }
        Err(_) => return Ok(Vec::new()),
    };

    let mut fonts = Vec::new();
    for (name_bytes, font_ref) in font_dict.iter() {
        let name = String::from_utf8_lossy(name_bytes).to_string();
        let (_, font_obj) = doc.dereference(font_ref).unwrap_or((None, font_ref));
        if let Ok(font_dict) = font_obj.as_dict() {
            match FontInfo::from_lopdf_dict(doc, &name, font_dict) {
                Ok(font) => fonts.push(font),
                Err(e) => {
                    log::warn!("Failed to parse font '{}': {}", name, e);
                }
            }
        }
    }

    Ok(fonts)
}

fn get_resources(
    doc: &lopdf::Document,
    page_dict: &lopdf::Dictionary,
) -> Option<lopdf::Dictionary> {
    if let Ok(res_ref) = page_dict.get(b"Resources") {
        let (_, obj) = doc.dereference(res_ref).unwrap_or((None, res_ref));
        if let Ok(dict) = obj.as_dict() {
            return Some(dict.clone());
        }
    }

    let mut current = page_dict.clone();
    loop {
        match current.get(b"Parent") {
            Ok(parent_ref) => {
                let (_, parent_obj) = doc.dereference(parent_ref).ok()?;
                let parent_dict = parent_obj.as_dict().ok()?;
                if let Ok(res_ref) = parent_dict.get(b"Resources") {
                    let (_, obj) = doc.dereference(res_ref).unwrap_or((None, res_ref));
                    if let Ok(dict) = obj.as_dict() {
                        return Some(dict.clone());
                    }
                }
                current = parent_dict.clone();
            }
            Err(_) => return None,
        }
    }
}
