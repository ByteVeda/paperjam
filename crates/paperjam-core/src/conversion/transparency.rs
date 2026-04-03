//! Transparency removal for PDF/A-1 conversion.

use lopdf::Object;

/// Remove transparency features forbidden by PDF/A-1.
///
/// - Removes /Group entries with /S /Transparency from pages
/// - Sets blend modes to Normal in ExtGState
/// - Removes /SMask from ExtGState
pub fn remove_transparency(
    doc: &mut lopdf::Document,
    page_map: &std::collections::BTreeMap<u32, lopdf::ObjectId>,
) -> Vec<String> {
    let mut actions = Vec::new();

    for (&page_num, &page_id) in page_map {
        // Remove /Group with /S /Transparency from page dict
        if let Ok(page_obj) = doc.get_object_mut(page_id) {
            if let Ok(page_dict) = page_obj.as_dict_mut() {
                let should_remove = matches!(
                    page_dict.get(b"Group"),
                    Ok(Object::Dictionary(d))
                        if matches!(d.get(b"S"), Ok(Object::Name(n)) if n == b"Transparency")
                );

                if should_remove {
                    page_dict.remove(b"Group");
                    actions.push(format!("Removed transparency group from page {}", page_num));
                }
            }
        }

        // Fix ExtGState objects referenced from page resources
        let gs_ids = collect_extgstate_ids(doc, page_id);
        for gs_id in gs_ids {
            if let Ok(gs_obj) = doc.get_object_mut(gs_id) {
                if let Ok(gs_dict) = gs_obj.as_dict_mut() {
                    // Fix blend mode
                    if let Ok(Object::Name(bm)) = gs_dict.get(b"BM") {
                        if bm != b"Normal" && bm != b"Compatible" {
                            let old_bm = String::from_utf8_lossy(bm).to_string();
                            gs_dict.set("BM", Object::Name(b"Normal".to_vec()));
                            actions.push(format!(
                                "Reset blend mode '{}' to Normal on page {}",
                                old_bm, page_num
                            ));
                        }
                    }

                    // Remove soft mask
                    let has_smask = if let Ok(smask) = gs_dict.get(b"SMask") {
                        !matches!(smask, Object::Name(n) if n == b"None")
                    } else {
                        false
                    };
                    if has_smask {
                        gs_dict.set("SMask", Object::Name(b"None".to_vec()));
                        actions.push(format!(
                            "Removed soft mask from ExtGState on page {}",
                            page_num
                        ));
                    }
                }
            }
        }
    }

    actions
}

/// Collect ObjectIds of all ExtGState objects referenced from a page's resources.
fn collect_extgstate_ids(doc: &lopdf::Document, page_id: lopdf::ObjectId) -> Vec<lopdf::ObjectId> {
    let mut ids = Vec::new();

    let page_dict = match doc.get_object(page_id).and_then(|o| o.as_dict().cloned()) {
        Ok(d) => d,
        Err(_) => return ids,
    };

    let resources = match page_dict.get(b"Resources") {
        Ok(Object::Dictionary(d)) => d.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => return ids,
            },
            Err(_) => return ids,
        },
        _ => return ids,
    };

    let gs_dict = match resources.get(b"ExtGState") {
        Ok(Object::Dictionary(d)) => d.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => return ids,
            },
            Err(_) => return ids,
        },
        _ => return ids,
    };

    for (_, gs_ref) in gs_dict.iter() {
        if let Object::Reference(id) = gs_ref {
            ids.push(*id);
        }
    }

    ids
}
