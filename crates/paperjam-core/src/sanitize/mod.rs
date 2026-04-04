use lopdf::{Object, ObjectId};

use crate::document::Document;
use crate::error::{PdfError, Result};

pub use paperjam_model::sanitize::*;

/// Sanitize a PDF document by removing potentially dangerous objects.
///
/// Returns a new sanitized Document and a result describing what was removed.
pub fn sanitize(doc: &Document, options: &SanitizeOptions) -> Result<(Document, SanitizeResult)> {
    let mut inner = doc.inner().clone();
    let mut result = SanitizeResult {
        javascript_removed: 0,
        embedded_files_removed: 0,
        actions_removed: 0,
        links_removed: 0,
        items: Vec::new(),
    };

    if options.remove_javascript {
        remove_javascript(&mut inner, &mut result);
    }

    if options.remove_embedded_files {
        remove_embedded_files(&mut inner, &mut result);
    }

    if options.remove_actions {
        remove_actions(&mut inner, &mut result);
    }

    if options.remove_links {
        remove_link_annotations(&mut inner, &mut result);
    }

    inner.renumber_objects();
    inner.adjust_zero_pages();

    let result_doc = Document::from_lopdf(inner).map_err(|e| {
        PdfError::Sanitize(format!("Failed to reconstruct sanitized document: {}", e))
    })?;

    Ok((result_doc, result))
}

// --- Internal helpers ---

/// Get the catalog object ID from the trailer.
fn get_catalog_id(doc: &lopdf::Document) -> Option<ObjectId> {
    doc.trailer
        .get(b"Root")
        .ok()
        .and_then(|r| r.as_reference().ok())
}

/// Remove JavaScript from the document.
fn remove_javascript(doc: &mut lopdf::Document, result: &mut SanitizeResult) {
    // 1. Remove /JS keys from all dictionaries (inline JavaScript)
    let all_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();
    for id in &all_ids {
        let has_js = if let Some(obj) = doc.objects.get(id) {
            match obj {
                Object::Dictionary(dict) => dict.get(b"JS").is_ok(),
                Object::Stream(stream) => stream.dict.get(b"JS").is_ok(),
                _ => false,
            }
        } else {
            false
        };

        if has_js {
            if let Some(obj) = doc.objects.get_mut(id) {
                match obj {
                    Object::Dictionary(dict) => {
                        dict.remove(b"JS");
                    }
                    Object::Stream(stream) => {
                        stream.dict.remove(b"JS");
                    }
                    _ => {}
                }
            }
            result.javascript_removed += 1;
            result.items.push(SanitizedItem {
                category: "javascript".into(),
                description: "Removed /JS key from object".into(),
                page: None,
            });
        }
    }

    // 2. Remove JavaScript actions (dicts with /S = /JavaScript)
    for id in &all_ids {
        let is_js_action = if let Some(obj) = doc.objects.get(id) {
            is_javascript_action(obj)
        } else {
            false
        };

        if is_js_action {
            // Nullify the action by removing /S and /JS keys
            if let Some(obj) = doc.objects.get_mut(id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.remove(b"S");
                    dict.remove(b"JS");
                }
            }
            result.javascript_removed += 1;
            result.items.push(SanitizedItem {
                category: "javascript".into(),
                description: "Removed JavaScript action".into(),
                page: None,
            });
        }
    }

    // 3. Remove /JavaScript name tree from the catalog's /Names dictionary
    if let Some(catalog_id) = get_catalog_id(doc) {
        let names_ref = get_dict_reference(doc, catalog_id, b"Names");
        if let Some(names_id) = names_ref {
            let has_js_tree = if let Some(obj) = doc.objects.get(&names_id) {
                obj.as_dict()
                    .map(|d| d.get(b"JavaScript").is_ok())
                    .unwrap_or(false)
            } else {
                false
            };

            if has_js_tree {
                // Collect all objects referenced by the JavaScript name tree
                let js_tree_refs = collect_name_tree_refs(doc, names_id, b"JavaScript");
                for ref_id in &js_tree_refs {
                    doc.objects.remove(ref_id);
                }

                // Remove the /JavaScript key from /Names
                if let Some(obj) = doc.objects.get_mut(&names_id) {
                    if let Ok(dict) = obj.as_dict_mut() {
                        dict.remove(b"JavaScript");
                    }
                }

                result.javascript_removed += 1;
                result.items.push(SanitizedItem {
                    category: "javascript".into(),
                    description: format!(
                        "Removed /JavaScript name tree ({} referenced objects)",
                        js_tree_refs.len()
                    ),
                    page: None,
                });
            }
        }
    }
}

/// Remove embedded files from the document.
fn remove_embedded_files(doc: &mut lopdf::Document, result: &mut SanitizeResult) {
    // 1. Remove /EmbeddedFiles from catalog's /Names
    if let Some(catalog_id) = get_catalog_id(doc) {
        let names_ref = get_dict_reference(doc, catalog_id, b"Names");
        if let Some(names_id) = names_ref {
            let has_embedded = if let Some(obj) = doc.objects.get(&names_id) {
                obj.as_dict()
                    .map(|d| d.get(b"EmbeddedFiles").is_ok())
                    .unwrap_or(false)
            } else {
                false
            };

            if has_embedded {
                let ef_refs = collect_name_tree_refs(doc, names_id, b"EmbeddedFiles");
                for ref_id in &ef_refs {
                    doc.objects.remove(ref_id);
                }

                if let Some(obj) = doc.objects.get_mut(&names_id) {
                    if let Ok(dict) = obj.as_dict_mut() {
                        dict.remove(b"EmbeddedFiles");
                    }
                }

                result.embedded_files_removed += 1;
                result.items.push(SanitizedItem {
                    category: "embedded_file".into(),
                    description: format!(
                        "Removed /EmbeddedFiles name tree ({} referenced objects)",
                        ef_refs.len()
                    ),
                    page: None,
                });
            }
        }

        // 2. Remove /AF (associated files) from catalog
        remove_dict_key(
            doc,
            catalog_id,
            b"AF",
            "embedded_file",
            "Removed /AF from catalog",
            result,
        );
    }

    // 3. Remove /FileAttachment annotations from all pages
    remove_annotations_by_subtype(doc, b"FileAttachment", "embedded_file", result);
}

/// Remove dangerous actions from the document.
fn remove_actions(doc: &mut lopdf::Document, result: &mut SanitizeResult) {
    // 1. Remove /OpenAction from catalog
    if let Some(catalog_id) = get_catalog_id(doc) {
        // Get the /OpenAction value first to remove referenced objects
        let open_action_ref = get_dict_reference(doc, catalog_id, b"OpenAction");
        if let Some(action_id) = open_action_ref {
            doc.objects.remove(&action_id);
        }

        let had_open_action = if let Some(obj) = doc.objects.get(&catalog_id) {
            obj.as_dict()
                .map(|d| d.get(b"OpenAction").is_ok())
                .unwrap_or(false)
        } else {
            false
        };

        if had_open_action {
            if let Some(obj) = doc.objects.get_mut(&catalog_id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.remove(b"OpenAction");
                }
            }
            result.actions_removed += 1;
            result.items.push(SanitizedItem {
                category: "action".into(),
                description: "Removed /OpenAction from catalog".into(),
                page: None,
            });
        }
    }

    // 2. Remove /AA (Additional Actions) from all objects
    let all_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();
    for id in &all_ids {
        let has_aa = if let Some(obj) = doc.objects.get(id) {
            match obj {
                Object::Dictionary(dict) => dict.get(b"AA").is_ok(),
                Object::Stream(stream) => stream.dict.get(b"AA").is_ok(),
                _ => false,
            }
        } else {
            false
        };

        if has_aa {
            // Collect referenced action objects from /AA before removing
            let aa_refs = get_dict_reference(doc, *id, b"AA");
            if let Some(aa_id) = aa_refs {
                doc.objects.remove(&aa_id);
            }

            if let Some(obj) = doc.objects.get_mut(id) {
                match obj {
                    Object::Dictionary(dict) => {
                        dict.remove(b"AA");
                    }
                    Object::Stream(stream) => {
                        stream.dict.remove(b"AA");
                    }
                    _ => {}
                }
            }
            result.actions_removed += 1;
            result.items.push(SanitizedItem {
                category: "action".into(),
                description: "Removed /AA (Additional Actions) from object".into(),
                page: None,
            });
        }
    }

    // 3. Remove dangerous action types: /Launch, /GoToR, /GoToE, /SubmitForm, /ImportData
    let dangerous_actions: &[&[u8]] = &[
        b"Launch",
        b"GoToR",
        b"GoToE",
        b"SubmitForm",
        b"ImportData",
        b"RichMediaExecute",
    ];
    for id in &all_ids {
        let is_dangerous = if let Some(obj) = doc.objects.get(id) {
            is_dangerous_action(obj, dangerous_actions)
        } else {
            false
        };

        if is_dangerous {
            let action_name = get_action_type_name(doc.objects.get(id));
            if let Some(obj) = doc.objects.get_mut(id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.remove(b"S");
                    dict.remove(b"F"); // file spec for Launch
                    dict.remove(b"Win");
                    dict.remove(b"Mac");
                    dict.remove(b"Unix");
                }
            }
            result.actions_removed += 1;
            result.items.push(SanitizedItem {
                category: "action".into(),
                description: format!("Removed dangerous action: /{}", action_name),
                page: None,
            });
        }
    }

    // 4. Remove /A (Action) from annotations that have dangerous actions
    for id in &all_ids {
        let has_dangerous_a = if let Some(obj) = doc.objects.get(id) {
            has_dangerous_annotation_action(doc, obj, dangerous_actions)
        } else {
            false
        };

        if has_dangerous_a {
            if let Some(obj) = doc.objects.get_mut(id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.remove(b"A");
                }
            }
            result.actions_removed += 1;
            result.items.push(SanitizedItem {
                category: "action".into(),
                description: "Removed dangerous /A action from annotation".into(),
                page: None,
            });
        }
    }
}

/// Remove link annotations from all pages.
fn remove_link_annotations(doc: &mut lopdf::Document, result: &mut SanitizeResult) {
    remove_annotations_by_subtype(doc, b"Link", "link", result);
}

// --- Low-level helpers ---

/// Check if an object is a JavaScript action dictionary.
fn is_javascript_action(obj: &Object) -> bool {
    let dict = match obj {
        Object::Dictionary(d) => d,
        Object::Stream(s) => &s.dict,
        _ => return false,
    };
    if let Ok(s) = dict.get(b"S") {
        if let Ok(name) = s.as_name_str() {
            return name == "JavaScript";
        }
    }
    false
}

/// Check if an object is a dangerous action type.
fn is_dangerous_action(obj: &Object, dangerous: &[&[u8]]) -> bool {
    let dict = match obj {
        Object::Dictionary(d) => d,
        Object::Stream(s) => &s.dict,
        _ => return false,
    };
    if let Ok(s) = dict.get(b"S") {
        if let Ok(name) = s.as_name() {
            return dangerous.contains(&name);
        }
    }
    false
}

/// Get the action type name from an object for logging.
fn get_action_type_name(obj: Option<&Object>) -> String {
    let obj = match obj {
        Some(o) => o,
        None => return "Unknown".into(),
    };
    let dict = match obj {
        Object::Dictionary(d) => d,
        Object::Stream(s) => &s.dict,
        _ => return "Unknown".into(),
    };
    dict.get(b"S")
        .ok()
        .and_then(|s| s.as_name_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".into())
}

/// Check if an annotation object has a dangerous /A action.
fn has_dangerous_annotation_action(
    doc: &lopdf::Document,
    obj: &Object,
    dangerous: &[&[u8]],
) -> bool {
    let dict = match obj {
        Object::Dictionary(d) => d,
        _ => return false,
    };

    // Must be an annotation (have /Subtype)
    if dict.get(b"Subtype").is_err() {
        return false;
    }

    // Check /A action
    if let Ok(action_ref) = dict.get(b"A") {
        let action_obj = match action_ref {
            Object::Reference(id) => doc.get_object(*id).ok(),
            obj => Some(obj),
        };
        if let Some(action) = action_obj {
            return is_dangerous_action(action, dangerous);
        }
    }

    false
}

/// Get a reference to an object stored under a key in a dictionary object.
fn get_dict_reference(doc: &lopdf::Document, dict_id: ObjectId, key: &[u8]) -> Option<ObjectId> {
    let obj = doc.objects.get(&dict_id)?;
    let dict = match obj {
        Object::Dictionary(d) => d,
        Object::Stream(s) => &s.dict,
        _ => return None,
    };
    let val = dict.get(key).ok()?;
    val.as_reference().ok()
}

/// Remove a key from a dict object, recording the sanitization.
fn remove_dict_key(
    doc: &mut lopdf::Document,
    dict_id: ObjectId,
    key: &[u8],
    category: &str,
    description: &str,
    result: &mut SanitizeResult,
) {
    let has_key = if let Some(obj) = doc.objects.get(&dict_id) {
        match obj {
            Object::Dictionary(d) => d.get(key).is_ok(),
            Object::Stream(s) => s.dict.get(key).is_ok(),
            _ => false,
        }
    } else {
        false
    };

    if has_key {
        if let Some(obj) = doc.objects.get_mut(&dict_id) {
            match obj {
                Object::Dictionary(d) => {
                    d.remove(key);
                }
                Object::Stream(s) => {
                    s.dict.remove(key);
                }
                _ => {}
            }
        }
        match category {
            "embedded_file" => result.embedded_files_removed += 1,
            "javascript" => result.javascript_removed += 1,
            "action" => result.actions_removed += 1,
            "link" => result.links_removed += 1,
            _ => {}
        }
        result.items.push(SanitizedItem {
            category: category.into(),
            description: description.into(),
            page: None,
        });
    }
}

/// Collect all object IDs referenced by a name tree entry.
fn collect_name_tree_refs(
    doc: &lopdf::Document,
    names_dict_id: ObjectId,
    tree_key: &[u8],
) -> Vec<ObjectId> {
    let mut refs = Vec::new();

    let tree_ref = if let Some(obj) = doc.objects.get(&names_dict_id) {
        if let Ok(dict) = obj.as_dict() {
            dict.get(tree_key).ok().and_then(|v| v.as_reference().ok())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(tree_id) = tree_ref {
        refs.push(tree_id);
        collect_name_tree_node_refs(doc, tree_id, &mut refs);
    }

    refs
}

/// Recursively collect references from a name tree node.
fn collect_name_tree_node_refs(doc: &lopdf::Document, node_id: ObjectId, refs: &mut Vec<ObjectId>) {
    let obj = match doc.objects.get(&node_id) {
        Some(o) => o,
        None => return,
    };
    let dict = match obj.as_dict() {
        Ok(d) => d,
        Err(_) => return,
    };

    // Process /Names array (leaf nodes): [name1, ref1, name2, ref2, ...]
    if let Ok(names) = dict.get(b"Names") {
        if let Ok(arr) = names.as_array() {
            // Values are at even indices starting from 1
            for item in arr.iter().skip(1).step_by(2) {
                if let Ok(id) = item.as_reference() {
                    refs.push(id);
                }
            }
        }
    }

    // Process /Kids array (intermediate nodes)
    if let Ok(kids) = dict.get(b"Kids") {
        if let Ok(arr) = kids.as_array() {
            for kid in arr {
                if let Ok(kid_id) = kid.as_reference() {
                    refs.push(kid_id);
                    collect_name_tree_node_refs(doc, kid_id, refs);
                }
            }
        }
    }
}

/// Remove annotations with a given /Subtype from all pages.
fn remove_annotations_by_subtype(
    doc: &mut lopdf::Document,
    subtype: &[u8],
    category: &str,
    result: &mut SanitizeResult,
) {
    let pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();

    for (page_num, page_id) in pages {
        // Read the annotation refs for this page
        let annot_refs = {
            let page_obj = match doc.objects.get(&page_id) {
                Some(o) => o,
                None => continue,
            };
            let page_dict = match page_obj.as_dict() {
                Ok(d) => d,
                Err(_) => continue,
            };

            let annots_obj = match page_dict.get(b"Annots") {
                Ok(obj) => obj.clone(),
                Err(_) => continue,
            };

            match &annots_obj {
                Object::Array(arr) => arr.clone(),
                Object::Reference(id) => match doc.get_object(*id) {
                    Ok(Object::Array(arr)) => arr.clone(),
                    _ => continue,
                },
                _ => continue,
            }
        };

        // Determine which annotations to keep and which to remove
        let mut kept = Vec::new();
        let mut removed_count = 0usize;

        for annot_ref in &annot_refs {
            let annot_id = match annot_ref.as_reference() {
                Ok(id) => id,
                Err(_) => {
                    kept.push(annot_ref.clone());
                    continue;
                }
            };

            let is_target = if let Ok(obj) = doc.get_object(annot_id) {
                if let Ok(dict) = obj.as_dict() {
                    dict.get(b"Subtype")
                        .ok()
                        .and_then(|s| s.as_name().ok())
                        .map(|name| name == subtype)
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            };

            if is_target {
                doc.objects.remove(&annot_id);
                removed_count += 1;
                result.items.push(SanitizedItem {
                    category: category.into(),
                    description: format!(
                        "Removed /{} annotation",
                        String::from_utf8_lossy(subtype)
                    ),
                    page: Some(page_num),
                });
            } else {
                kept.push(annot_ref.clone());
            }
        }

        if removed_count > 0 {
            // Update the page's /Annots array
            if let Some(obj) = doc.objects.get_mut(&page_id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    if kept.is_empty() {
                        dict.remove(b"Annots");
                    } else {
                        dict.set("Annots", Object::Array(kept));
                    }
                }
            }

            match category {
                "embedded_file" => result.embedded_files_removed += removed_count,
                "link" => result.links_removed += removed_count,
                _ => result.actions_removed += removed_count,
            }
        }
    }
}
