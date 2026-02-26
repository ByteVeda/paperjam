pub mod appearance;
pub mod create;
pub mod traverse;
pub mod types;

use std::collections::HashMap;

use lopdf::{dictionary, Object};

use crate::document::Document;
use crate::error::{PdfError, Result};
use types::{FillFormOptions, FillFormResult, FormField, ModifyFieldOptions, ModifyFieldResult};

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

    // First extract fields to get name -> field info mapping
    let fields = extract_form_fields(&inner)?;

    // Build a name -> (read_only, field_type) map for lookup
    let field_name_map: HashMap<String, (bool, types::FormFieldType)> = fields
        .iter()
        .map(|f| (f.name.clone(), (f.read_only, f.field_type.clone())))
        .collect();

    let mut filled = 0usize;
    let mut not_found = Vec::new();
    let mut filled_fields: Vec<(String, String, types::FormFieldType)> = Vec::new();

    for (name, value) in values {
        match field_name_map.get(name) {
            None => {
                not_found.push(name.clone());
            }
            Some((true, _)) => {
                // Read-only field, skip
                not_found.push(name.clone());
            }
            Some((false, field_type)) => {
                // Find and update the field object
                if set_field_value(&mut inner, name, value, field_type)? {
                    filled += 1;
                    filled_fields.push((name.clone(), value.clone(), field_type.clone()));
                } else {
                    not_found.push(name.clone());
                }
            }
        }
    }

    // Generate appearance streams if requested
    if options.generate_appearances && filled > 0 {
        for (name, value, field_type) in &filled_fields {
            if let Some(field_id) = traverse::find_field_id(&inner, name)? {
                appearance::generate_field_appearance(&mut inner, field_id, field_type, value)?;
            }
        }
        // Remove NeedAppearances since we generated explicit appearances
        remove_need_appearances(&mut inner)?;
    } else if options.need_appearances && filled > 0 {
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
/// Type-aware: uses Name objects for buttons, String for text/choice fields.
fn set_field_value(
    doc: &mut lopdf::Document,
    target_name: &str,
    value: &str,
    field_type: &types::FormFieldType,
) -> Result<bool> {
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

    set_field_in_tree(doc, &fields_array, "", target_name, value, field_type)
}

/// Recursively search the field tree for a field by name and set its value.
///
/// Type-aware: for Checkbox/RadioButton fields, uses Name objects and sets /AS
/// on widget annotations. For text/choice fields, uses String objects.
fn set_field_in_tree(
    doc: &mut lopdf::Document,
    field_refs: &[Object],
    parent_name: &str,
    target_name: &str,
    value: &str,
    field_type: &types::FormFieldType,
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
                if set_field_in_tree(
                    doc,
                    &kids_clone,
                    &fq_name,
                    target_name,
                    value,
                    field_type,
                )? {
                    return Ok(true);
                }
            } else if fq_name == target_name {
                // This is the target field with widget kids
                set_value_on_field(doc, field_id, value, field_type)?;
                // For button types with widget kids, update /AS on widgets
                if matches!(
                    field_type,
                    types::FormFieldType::Checkbox | types::FormFieldType::RadioButton
                ) {
                    set_as_on_widget_kids(doc, &kids_clone, value)?;
                }
                return Ok(true);
            }
        } else if fq_name == target_name {
            // Terminal field (merged field+widget), set value and /AS
            set_value_on_field(doc, field_id, value, field_type)?;
            if matches!(
                field_type,
                types::FormFieldType::Checkbox | types::FormFieldType::RadioButton
            ) {
                // Terminal merged widget — set /AS on itself
                let field_obj = doc
                    .get_object_mut(field_id)
                    .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?;
                let field_dict = field_obj
                    .as_dict_mut()
                    .map_err(|_| PdfError::Form("Field not dict".to_string()))?;
                field_dict.set("AS", Object::Name(value.as_bytes().to_vec()));
            }
            return Ok(true);
        }
    }

    Ok(false)
}

/// Set /V on a field, using the appropriate object type for the field type.
fn set_value_on_field(
    doc: &mut lopdf::Document,
    field_id: lopdf::ObjectId,
    value: &str,
    field_type: &types::FormFieldType,
) -> Result<()> {
    let field_obj = doc
        .get_object_mut(field_id)
        .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?;
    let field_dict = field_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("Field is not a dictionary".to_string()))?;

    match field_type {
        types::FormFieldType::Checkbox | types::FormFieldType::RadioButton => {
            field_dict.set("V", Object::Name(value.as_bytes().to_vec()));
        }
        _ => {
            field_dict.set(
                "V",
                Object::String(value.as_bytes().to_vec(), lopdf::StringFormat::Literal),
            );
        }
    }
    Ok(())
}

/// Set /AS on widget kids based on the selected value.
///
/// For checkboxes: matching widget gets /AS=Name(value), others get /AS=Name("Off").
/// For radio buttons: walk kids, check each widget's /AP/N dict keys to find the
/// matching export value. Set that widget's /AS to the export value, others to "Off".
fn set_as_on_widget_kids(
    doc: &mut lopdf::Document,
    kids: &[Object],
    value: &str,
) -> Result<()> {
    // First pass: collect widget IDs and their export values
    let widget_info: Vec<(lopdf::ObjectId, Option<String>)> = kids
        .iter()
        .filter_map(|k| k.as_reference().ok())
        .map(|kid_id| {
            let export_val = get_widget_export_value_for_as(doc, kid_id);
            (kid_id, export_val)
        })
        .collect();

    // Second pass: set /AS on each widget
    for (kid_id, export_val) in widget_info {
        let as_value = if let Some(ref ev) = export_val {
            if ev == value {
                value.to_string()
            } else {
                "Off".to_string()
            }
        } else {
            // No export value found — for checkbox, if value != "Off" set it, else "Off"
            if value != "Off" {
                value.to_string()
            } else {
                "Off".to_string()
            }
        };

        let kid_obj = doc
            .get_object_mut(kid_id)
            .map_err(|e| PdfError::Form(format!("Cannot get widget kid: {}", e)))?;
        let kid_dict = kid_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("Widget kid not dict".to_string()))?;
        kid_dict.set("AS", Object::Name(as_value.as_bytes().to_vec()));
    }

    Ok(())
}

/// Get the export value of a widget from /AP/N dict keys (the non-"Off" key).
fn get_widget_export_value_for_as(doc: &lopdf::Document, widget_id: lopdf::ObjectId) -> Option<String> {
    let obj = doc.get_object(widget_id).ok()?;
    let dict = obj.as_dict().ok()?;
    let ap = dict.get(b"AP").ok()?;
    let ap_dict = match ap {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
        _ => return None,
    };
    let n = ap_dict.get(b"N").ok()?;
    let n_dict = match n {
        Object::Dictionary(d) => d,
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok()?,
        _ => return None,
    };

    for (key, _) in n_dict.iter() {
        if key != b"Off" {
            return Some(String::from_utf8_lossy(key).to_string());
        }
    }
    None
}

/// Modify properties of a form field by name.
pub fn modify_form_field(
    doc: &Document,
    field_name: &str,
    options: &ModifyFieldOptions,
) -> Result<(Document, ModifyFieldResult)> {
    let mut inner = doc.inner().clone();

    let field_id = match traverse::find_field_id(&inner, field_name)? {
        Some(id) => id,
        None => {
            return Ok((
                Document::from_lopdf(inner)?,
                ModifyFieldResult {
                    field_name: field_name.to_string(),
                    modified: false,
                },
            ));
        }
    };

    let field_obj = inner
        .get_object_mut(field_id)
        .map_err(|e| PdfError::Form(format!("Cannot get field: {}", e)))?;
    let field_dict = field_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("Field is not a dictionary".to_string()))?;

    // Update value
    if let Some(ref value) = options.value {
        field_dict.set(
            "V",
            Object::String(value.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }

    // Update default value
    if let Some(ref dv) = options.default_value {
        field_dict.set(
            "DV",
            Object::String(dv.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }

    // Update read_only / required via /Ff flags
    if options.read_only.is_some() || options.required.is_some() {
        let current_ff = field_dict
            .get(b"Ff")
            .ok()
            .and_then(|o| match o {
                Object::Integer(v) => Some(*v as u32),
                _ => None,
            })
            .unwrap_or(0);

        let mut ff = current_ff;
        if let Some(ro) = options.read_only {
            if ro {
                ff |= 1;
            } else {
                ff &= !1;
            }
        }
        if let Some(req) = options.required {
            if req {
                ff |= 2;
            } else {
                ff &= !2;
            }
        }
        field_dict.set("Ff", Object::Integer(ff as i64));
    }

    // Update max length
    if let Some(max_len) = options.max_length {
        field_dict.set("MaxLen", Object::Integer(max_len as i64));
    }

    // Update options
    if let Some(ref opts) = options.options {
        let opt_array: Vec<Object> = opts
            .iter()
            .map(|o| {
                Object::Array(vec![
                    Object::String(
                        o.export_value.as_bytes().to_vec(),
                        lopdf::StringFormat::Literal,
                    ),
                    Object::String(
                        o.display.as_bytes().to_vec(),
                        lopdf::StringFormat::Literal,
                    ),
                ])
            })
            .collect();
        field_dict.set("Opt", Object::Array(opt_array));
    }

    Ok((
        Document::from_lopdf(inner)?,
        ModifyFieldResult {
            field_name: field_name.to_string(),
            modified: true,
        },
    ))
}

/// Ensure AcroForm exists in the document, returning its ObjectId.
pub(crate) fn ensure_acroform(doc: &mut lopdf::Document) -> Result<lopdf::ObjectId> {
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

    // Check if AcroForm already exists as a reference
    if let Ok(obj) = root_dict.get(b"AcroForm") {
        if let Ok(af_id) = obj.as_reference() {
            return Ok(af_id);
        }
    }

    // Create new AcroForm
    let acroform = dictionary! {
        "Fields" => Object::Array(Vec::new()),
    };
    let af_id = doc.new_object_id();
    doc.objects.insert(af_id, Object::Dictionary(acroform));

    let root_obj = doc
        .get_object_mut(root_id)
        .map_err(|e| PdfError::Form(format!("Cannot get root: {}", e)))?;
    let root_dict = root_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("/Root not dict".to_string()))?;
    root_dict.set("AcroForm", Object::Reference(af_id));

    Ok(af_id)
}

/// Add a field reference to the AcroForm /Fields array.
pub(crate) fn add_field_to_acroform(
    doc: &mut lopdf::Document,
    field_id: lopdf::ObjectId,
) -> Result<()> {
    let af_id = ensure_acroform(doc)?;

    let af_obj = doc
        .get_object_mut(af_id)
        .map_err(|e| PdfError::Form(format!("Cannot get AcroForm: {}", e)))?;
    let af_dict = af_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Form("AcroForm not dict".to_string()))?;

    match af_dict.get_mut(b"Fields") {
        Ok(Object::Array(fields)) => {
            fields.push(Object::Reference(field_id));
        }
        _ => {
            af_dict.set(
                "Fields",
                Object::Array(vec![Object::Reference(field_id)]),
            );
        }
    }

    Ok(())
}

/// Remove /NeedAppearances from the AcroForm dictionary.
fn remove_need_appearances(doc: &mut lopdf::Document) -> Result<()> {
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

    let acroform_ref = root_dict.get(b"AcroForm").ok().and_then(|o| match o {
        Object::Reference(id) => Some(*id),
        _ => None,
    });

    if let Some(af_id) = acroform_ref {
        let af_obj = doc
            .get_object_mut(af_id)
            .map_err(|e| PdfError::Form(format!("Cannot get AcroForm: {}", e)))?;
        let af_dict = af_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("AcroForm not dict".to_string()))?;
        af_dict.remove(b"NeedAppearances");
    } else {
        let root_obj = doc
            .get_object_mut(root_id)
            .map_err(|e| PdfError::Form(format!("Cannot get root: {}", e)))?;
        let root_dict = root_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Form("/Root not dict".to_string()))?;
        if let Ok(Object::Dictionary(af_dict)) = root_dict.get_mut(b"AcroForm") {
            af_dict.remove(b"NeedAppearances");
        }
    }

    Ok(())
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
