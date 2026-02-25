pub mod traverse;
pub mod types;

use std::collections::HashMap;

use lopdf::Object;

use crate::document::Document;
use crate::error::{PdfError, Result};
use types::{FillFormOptions, FillFormResult, FormField};

/// Check whether the document contains an interactive form (AcroForm).
pub fn has_form(doc: &lopdf::Document) -> bool {
    doc.trailer
        .get(b"Root")
        .ok()
        .and_then(|r| r.as_reference().ok())
        .and_then(|root_id| doc.get_object(root_id).ok())
        .and_then(|root| root.as_dict().ok())
        .and_then(|root_dict| root_dict.get(b"AcroForm").ok())
        .is_some()
}

/// Extract all form fields from the document's AcroForm.
pub fn extract_form_fields(doc: &lopdf::Document) -> Result<Vec<FormField>> {
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
        Err(_) => return Ok(Vec::new()), // No form
    };

    let acroform_dict = match acroform_obj {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc
            .get_object(*id)
            .map_err(|e| PdfError::Form(format!("Cannot dereference AcroForm: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Form("AcroForm is not a dictionary".to_string()))?,
        _ => return Ok(Vec::new()),
    };

    let fields_array = match acroform_dict.get(b"Fields") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(Vec::new()),
        },
        _ => return Ok(Vec::new()),
    };

    let annot_page_map = traverse::build_annot_page_map(doc);

    traverse::walk_fields(doc, &fields_array, "", None, 0, &annot_page_map)
}

/// Fill form fields by name, returning a new document and statistics.
///
/// Updates the /V (value) entry on each matching field. Sets /NeedAppearances
/// on the AcroForm so that PDF viewers regenerate appearances.
pub fn fill_form_fields(
    doc: &Document,
    values: &HashMap<String, String>,
    options: &FillFormOptions,
) -> Result<(Document, FillFormResult)> {
    let mut inner = doc.inner().clone();

    // First extract fields to get name -> ObjectId mapping
    let fields = extract_form_fields(&inner)?;

    // Build a name -> field info map for lookup
    let field_name_map: HashMap<String, bool> = fields
        .iter()
        .map(|f| (f.name.clone(), f.read_only))
        .collect();

    let mut filled = 0usize;
    let mut not_found = Vec::new();

    for (name, value) in values {
        match field_name_map.get(name) {
            None => {
                not_found.push(name.clone());
            }
            Some(true) => {
                // Read-only field, skip
                not_found.push(name.clone());
            }
            Some(false) => {
                // Find and update the field object
                if set_field_value(&mut inner, name, value)? {
                    filled += 1;
                } else {
                    not_found.push(name.clone());
                }
            }
        }
    }

    // Set /NeedAppearances if requested
    if options.need_appearances && filled > 0 {
        set_need_appearances(&mut inner)?;
    }

    let result = FillFormResult {
        fields_filled: filled,
        fields_not_found: not_found.len(),
        not_found_names: not_found,
    };

    let new_doc = Document::from_lopdf(inner)?;
    Ok((new_doc, result))
}

/// Set the value of a form field by fully-qualified name.
///
/// Walks the AcroForm /Fields tree to find the matching field and update /V.
fn set_field_value(doc: &mut lopdf::Document, target_name: &str, value: &str) -> Result<bool> {
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
        Ok(obj) => obj.clone(),
        Err(_) => return Ok(false),
    };

    let acroform_dict = match &acroform_obj {
        Object::Dictionary(d) => d.clone(),
        Object::Reference(id) => doc
            .get_object(*id)
            .map_err(|e| PdfError::Form(format!("Cannot dereference AcroForm: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Form("AcroForm is not a dictionary".to_string()))?
            .clone(),
        _ => return Ok(false),
    };

    let fields_array = match acroform_dict.get(b"Fields") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(false),
        },
        _ => return Ok(false),
    };

    set_field_in_tree(doc, &fields_array, "", target_name, value)
}

/// Recursively search the field tree for a field by name and set its value.
fn set_field_in_tree(
    doc: &mut lopdf::Document,
    field_refs: &[Object],
    parent_name: &str,
    target_name: &str,
    value: &str,
) -> Result<bool> {
    for field_ref in field_refs {
        let field_id = match field_ref.as_reference() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let dict = match doc.get_object(field_id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        // Get partial field name
        let partial_name = dict
            .get(b"T")
            .ok()
            .and_then(|o| match o {
                Object::String(bytes, _) => {
                    Some(String::from_utf8_lossy(bytes).to_string())
                }
                _ => None,
            })
            .unwrap_or_default();

        let fq_name = if parent_name.is_empty() {
            partial_name.clone()
        } else if partial_name.is_empty() {
            parent_name.to_string()
        } else {
            format!("{}.{}", parent_name, partial_name)
        };

        // Check if this node has kids
        if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
            let kids_clone = kids.clone();

            // Check if kids are sub-fields (have /T) or widget annotations
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
                // Recurse into sub-fields
                if set_field_in_tree(doc, &kids_clone, &fq_name, target_name, value)? {
                    return Ok(true);
                }
            } else if fq_name == target_name {
                // This is the target field with widget kids
                let field_obj = doc
                    .get_object_mut(field_id)
                    .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?;
                let field_dict = field_obj
                    .as_dict_mut()
                    .map_err(|_| PdfError::Form("Field is not a dictionary".to_string()))?;
                field_dict.set(
                    "V",
                    Object::String(
                        value.as_bytes().to_vec(),
                        lopdf::StringFormat::Literal,
                    ),
                );
                return Ok(true);
            }
        } else if fq_name == target_name {
            // Terminal field, set the value
            let field_obj = doc
                .get_object_mut(field_id)
                .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?;
            let field_dict = field_obj
                .as_dict_mut()
                .map_err(|_| PdfError::Form("Field is not a dictionary".to_string()))?;
            field_dict.set(
                "V",
                Object::String(
                    value.as_bytes().to_vec(),
                    lopdf::StringFormat::Literal,
                ),
            );
            return Ok(true);
        }
    }

    Ok(false)
}

/// Set /NeedAppearances = true on the AcroForm dictionary.
fn set_need_appearances(doc: &mut lopdf::Document) -> Result<()> {
    let root_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Form("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Form("/Root not ref".to_string()))?;

    let root_dict = doc
        .get_object(root_id)
        .map_err(|e| PdfError::Form(format!("Cannot get root: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Form("/Root not dict".to_string()))?;

    // Get AcroForm reference if indirect, or it's inline
    let acroform_ref = root_dict.get(b"AcroForm").ok().and_then(|o| match o {
        Object::Reference(id) => Some(*id),
        _ => None,
    });

    if let Some(af_id) = acroform_ref {
        // AcroForm is an indirect object
        let af_obj = doc
            .get_object_mut(af_id)
            .map_err(|e| PdfError::Form(format!("Cannot get AcroForm: {}", e)))?;
        let af_dict = af_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("AcroForm not dict".to_string()))?;
        af_dict.set("NeedAppearances", Object::Boolean(true));
    } else {
        // AcroForm is inline in the root
        let root_obj = doc
            .get_object_mut(root_id)
            .map_err(|e| PdfError::Form(format!("Cannot get root: {}", e)))?;
        let root_dict = root_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("/Root not dict".to_string()))?;
        if let Ok(Object::Dictionary(af_dict)) = root_dict.get_mut(b"AcroForm") {
            af_dict.set("NeedAppearances", Object::Boolean(true));
        }
    }

    Ok(())
}
