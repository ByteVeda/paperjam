use std::collections::BTreeMap;

use lopdf::{Object, ObjectId};

use crate::error::{PdfError, Result};
use crate::forms::types::{ChoiceOption, FormField, FormFieldType};

/// Build a map from annotation ObjectId -> page number (1-based).
pub fn build_annot_page_map(doc: &lopdf::Document) -> BTreeMap<ObjectId, u32> {
    let mut map = BTreeMap::new();
    let pages = doc.get_pages();
    for (&page_num, &page_id) in &pages {
        if let Ok(page_obj) = doc.get_object(page_id) {
            if let Ok(dict) = page_obj.as_dict() {
                if let Ok(annots) = dict.get(b"Annots") {
                    let annot_array = match annots {
                        Object::Array(arr) => arr.clone(),
                        Object::Reference(id) => {
                            match doc.get_object(*id) {
                                Ok(Object::Array(arr)) => arr.clone(),
                                _ => continue,
                            }
                        }
                        _ => continue,
                    };
                    for annot_ref in &annot_array {
                        if let Ok(id) = annot_ref.as_reference() {
                            map.insert(id, page_num);
                        }
                    }
                }
            }
        }
    }
    map
}

/// Recursively walk the AcroForm field tree, collecting FormField entries.
///
/// PDF form fields are organized in a tree:
/// - Non-terminal nodes have /Kids and optional /T (partial name)
/// - Terminal nodes (widget annotations) have /FT (field type) and /T
/// - /FT and /Ff flags are inherited from parent nodes
pub fn walk_fields(
    doc: &lopdf::Document,
    field_refs: &[Object],
    parent_name: &str,
    inherited_ft: Option<&[u8]>,
    inherited_ff: u32,
    annot_page_map: &BTreeMap<ObjectId, u32>,
) -> Result<Vec<FormField>> {
    let mut fields = Vec::new();

    for field_ref in field_refs {
        let field_id = match field_ref.as_reference() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let field_obj = doc
            .get_object(field_id)
            .map_err(|e| PdfError::Form(format!("Cannot get field object: {}", e)))?;
        let dict = match field_obj.as_dict() {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Get partial field name
        let partial_name = dict
            .get(b"T")
            .ok()
            .and_then(|o| obj_to_string(o, doc))
            .unwrap_or_default();

        // Build fully qualified name
        let fq_name = if parent_name.is_empty() {
            partial_name.clone()
        } else if partial_name.is_empty() {
            parent_name.to_string()
        } else {
            format!("{}.{}", parent_name, partial_name)
        };

        // Inherit or override field type
        let ft = dict
            .get(b"FT")
            .ok()
            .and_then(|o| match o {
                Object::Name(n) => Some(n.as_slice()),
                _ => None,
            })
            .or(inherited_ft);

        // Inherit or override flags
        let ff = dict
            .get(b"Ff")
            .ok()
            .and_then(obj_to_i64)
            .map(|v| v as u32)
            .unwrap_or(inherited_ff);

        // Check if this node has kids (non-terminal)
        if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
            // Check if kids are widget annotations or field nodes
            let kids_are_widgets = kids.iter().any(|k| {
                if let Ok(kid_id) = k.as_reference() {
                    if let Ok(kid_obj) = doc.get_object(kid_id) {
                        if let Ok(kid_dict) = kid_obj.as_dict() {
                            // Widget annotations have /Subtype = /Widget
                            if let Ok(Object::Name(subtype)) = kid_dict.get(b"Subtype") {
                                return subtype == b"Widget";
                            }
                            // If kid has no /T it's likely a widget, not a sub-field
                            return kid_dict.get(b"T").is_err();
                        }
                    }
                }
                false
            });

            if kids_are_widgets {
                // Terminal field with multiple widget annotations (e.g., radio buttons)
                let field_type = classify_field(ft, ff);
                let value = extract_value(dict, doc);
                let default_value = extract_default_value(dict, doc);
                let options = extract_options(dict, doc);
                let max_length = extract_max_length(dict);
                let read_only = (ff & 1) != 0;
                let required = (ff & 2) != 0;

                // Try to get page/rect from first widget kid
                let (page, rect) = kids
                    .first()
                    .and_then(|k| k.as_reference().ok())
                    .map(|kid_id| {
                        let page = annot_page_map.get(&kid_id).copied();
                        let rect = doc
                            .get_object(kid_id)
                            .ok()
                            .and_then(|o| o.as_dict().ok())
                            .and_then(extract_rect);
                        (page, rect)
                    })
                    .unwrap_or((None, None));

                fields.push(FormField {
                    name: fq_name,
                    field_type,
                    value,
                    default_value,
                    page,
                    rect,
                    read_only,
                    required,
                    options,
                    max_length,
                });
            } else {
                // Non-terminal: recurse into kids
                let child_fields = walk_fields(
                    doc,
                    kids,
                    &fq_name,
                    ft,
                    ff,
                    annot_page_map,
                )?;
                fields.extend(child_fields);
            }
        } else {
            // Terminal field (single widget annotation, or merged field+widget)
            let field_type = classify_field(ft, ff);
            let value = extract_value(dict, doc);
            let default_value = extract_default_value(dict, doc);
            let options = extract_options(dict, doc);
            let max_length = extract_max_length(dict);
            let read_only = (ff & 1) != 0;
            let required = (ff & 2) != 0;

            let page = annot_page_map.get(&field_id).copied();
            let rect = extract_rect(dict);

            fields.push(FormField {
                name: fq_name,
                field_type,
                value,
                default_value,
                page,
                rect,
                read_only,
                required,
                options,
                max_length,
            });
        }
    }

    Ok(fields)
}

/// Find a field's ObjectId by fully-qualified name.
///
/// Walks the AcroForm field tree looking for a field matching `target_name`.
/// Returns `Ok(Some(id))` if found, `Ok(None)` if not found.
pub(crate) fn find_field_id(
    doc: &lopdf::Document,
    target_name: &str,
) -> Result<Option<ObjectId>> {
    let root_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Form("No /Root in trailer".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Form("/Root is not a reference".to_string()))?;

    let root_dict = doc
        .get_object(root_id)
        .map_err(|e| PdfError::Form(format!("Cannot get root: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Form("/Root is not a dictionary".to_string()))?;

    let acroform_obj = match root_dict.get(b"AcroForm") {
        Ok(obj) => obj,
        Err(_) => return Ok(None),
    };

    let acroform_dict = match acroform_obj {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc
            .get_object(*id)
            .map_err(|e| PdfError::Form(format!("Cannot dereference AcroForm: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Form("AcroForm is not a dictionary".to_string()))?,
        _ => return Ok(None),
    };

    let fields_array = match acroform_dict.get(b"Fields") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };

    find_field_id_in_tree(doc, &fields_array, "", target_name)
}

fn find_field_id_in_tree(
    doc: &lopdf::Document,
    field_refs: &[Object],
    parent_name: &str,
    target_name: &str,
) -> Result<Option<ObjectId>> {
    for field_ref in field_refs {
        let field_id = match field_ref.as_reference() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let dict = match doc.get_object(field_id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let partial_name = dict
            .get(b"T")
            .ok()
            .and_then(|o| obj_to_string(o, doc))
            .unwrap_or_default();

        let fq_name = if parent_name.is_empty() {
            partial_name.clone()
        } else if partial_name.is_empty() {
            parent_name.to_string()
        } else {
            format!("{}.{}", parent_name, partial_name)
        };

        if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
            let kids_clone = kids.clone();

            let has_subfields = kids_clone.iter().any(|k| {
                if let Ok(kid_id) = k.as_reference() {
                    if let Ok(kid_obj) = doc.get_object(kid_id) {
                        if let Ok(kid_dict) = kid_obj.as_dict() {
                            return kid_dict.get(b"T").is_ok();
                        }
                    }
                }
                false
            });

            if has_subfields {
                if let Some(id) = find_field_id_in_tree(doc, &kids_clone, &fq_name, target_name)? {
                    return Ok(Some(id));
                }
            } else if fq_name == target_name {
                return Ok(Some(field_id));
            }
        } else if fq_name == target_name {
            return Ok(Some(field_id));
        }
    }

    Ok(None)
}

/// Classify a form field based on /FT name and /Ff flags.
pub(crate) fn classify_field(ft: Option<&[u8]>, ff: u32) -> FormFieldType {
    match ft {
        Some(b"Tx") => FormFieldType::Text,
        Some(b"Btn") => {
            if (ff & (1 << 16)) != 0 {
                FormFieldType::PushButton
            } else if (ff & (1 << 15)) != 0 {
                FormFieldType::RadioButton
            } else {
                FormFieldType::Checkbox
            }
        }
        Some(b"Ch") => {
            if (ff & (1 << 17)) != 0 {
                FormFieldType::ComboBox
            } else {
                FormFieldType::ListBox
            }
        }
        Some(b"Sig") => FormFieldType::Signature,
        Some(other) => FormFieldType::Unknown(String::from_utf8_lossy(other).to_string()),
        None => FormFieldType::Unknown("unknown".to_string()),
    }
}

/// Extract the current value (/V) from a field dictionary.
fn extract_value(dict: &lopdf::Dictionary, doc: &lopdf::Document) -> Option<String> {
    dict.get(b"V").ok().and_then(|o| match o {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(extract_obj_string),
        _ => None,
    })
}

/// Extract the default value (/DV) from a field dictionary.
fn extract_default_value(dict: &lopdf::Dictionary, doc: &lopdf::Document) -> Option<String> {
    dict.get(b"DV").ok().and_then(|o| match o {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(extract_obj_string),
        _ => None,
    })
}

/// Extract choice options (/Opt) from a field dictionary.
fn extract_options(dict: &lopdf::Dictionary, doc: &lopdf::Document) -> Vec<ChoiceOption> {
    let opt_obj = match dict.get(b"Opt") {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let opt_array = match opt_obj {
        Object::Array(arr) => arr,
        Object::Reference(id) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr,
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };

    opt_array
        .iter()
        .filter_map(|item| match item {
            // Simple option: just a string
            Object::String(bytes, _) => {
                let s = String::from_utf8_lossy(bytes).to_string();
                Some(ChoiceOption {
                    display: s.clone(),
                    export_value: s,
                })
            }
            // Array option: [export_value, display_text]
            Object::Array(pair) if pair.len() == 2 => {
                let export_value = extract_obj_string(&pair[0]).unwrap_or_default();
                let display = extract_obj_string(&pair[1]).unwrap_or_default();
                Some(ChoiceOption {
                    display,
                    export_value,
                })
            }
            _ => None,
        })
        .collect()
}

/// Extract max length (/MaxLen) from a text field.
fn extract_max_length(dict: &lopdf::Dictionary) -> u32 {
    dict.get(b"MaxLen")
        .ok()
        .and_then(obj_to_i64)
        .map(|v| v as u32)
        .unwrap_or(0)
}

/// Extract rect from a dictionary.
fn extract_rect(dict: &lopdf::Dictionary) -> Option<[f64; 4]> {
    match dict.get(b"Rect") {
        Ok(Object::Array(arr)) if arr.len() == 4 => {
            let mut r = [0.0f64; 4];
            for (i, v) in arr.iter().enumerate() {
                r[i] = obj_to_f64(v).unwrap_or(0.0);
            }
            Some(r)
        }
        _ => None,
    }
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

fn obj_to_i64(obj: &Object) -> Option<i64> {
    match obj {
        Object::Integer(v) => Some(*v),
        Object::Real(v) => Some(*v as i64),
        _ => None,
    }
}

fn obj_to_string(obj: &Object, doc: &lopdf::Document) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| obj_to_string(o, doc)),
        _ => None,
    }
}

fn extract_obj_string(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        _ => None,
    }
}
