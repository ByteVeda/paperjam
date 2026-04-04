use crate::document::Document;
use crate::error::{PdfError, Result};
use std::path::Path;

use lopdf::{dictionary, Object};

pub use paperjam_model::manipulation::MergeOptions;

/// Recursively remap all Object::Reference IDs by adding an offset.
fn remap_refs(object: &mut Object, offset: u32) {
    match object {
        Object::Reference(id) => {
            id.0 += offset;
        }
        Object::Array(arr) => {
            for item in arr.iter_mut() {
                remap_refs(item, offset);
            }
        }
        Object::Dictionary(dict) => {
            for (_, item) in dict.iter_mut() {
                remap_refs(item, offset);
            }
        }
        Object::Stream(stream) => {
            for (_, item) in stream.dict.iter_mut() {
                remap_refs(item, offset);
            }
        }
        _ => {}
    }
}

/// Merge multiple PDF documents into a single document.
///
/// Uses lopdf's object manipulation to combine page trees.
/// Preserves form fields from all documents, prefixing field names from
/// document N (2-based) with "docN_" to avoid collisions.
pub fn merge(documents: Vec<Document>, _options: &MergeOptions) -> Result<Document> {
    if documents.is_empty() {
        return Err(PdfError::Structure("Cannot merge zero documents".into()));
    }

    let mut inners: Vec<lopdf::Document> = documents.into_iter().map(|d| d.into_inner()).collect();
    let mut target = inners.remove(0);

    for (doc_idx, other) in inners.into_iter().enumerate() {
        // Manually merge: renumber objects in `other` to avoid collisions,
        // then copy objects and append pages.
        let max_id = target.max_id;
        let other_pages = other.get_pages();

        // Capture the other doc's catalog root ref before moving objects
        let other_root_id = other
            .trailer
            .get(b"Root")
            .ok()
            .and_then(|r| r.as_reference().ok())
            .map(|id| (id.0 + max_id, id.1));

        // Collect the new page references we need to add
        let new_page_refs: Vec<lopdf::ObjectId> = other_pages
            .values()
            .map(|page_id| (page_id.0 + max_id, page_id.1))
            .collect();

        // Find the target Pages root ID before mutable borrows
        let target_pages_id = target
            .catalog()
            .ok()
            .and_then(|cat| cat.get(b"Pages").ok())
            .and_then(|p| p.as_reference().ok());

        // Copy all objects with renumbered IDs and remapped internal references
        for (id, mut object) in other.objects {
            remap_refs(&mut object, max_id);
            let new_id = (id.0 + max_id, id.1);
            target.objects.insert(new_id, object);
        }

        // Update Parent references in copied pages to point to target's Pages root
        if let Some(pages_id) = target_pages_id {
            for &page_ref in &new_page_refs {
                if let Some(page_obj) = target.objects.get_mut(&page_ref) {
                    if let Ok(dict) = page_obj.as_dict_mut() {
                        dict.set("Parent", Object::Reference(pages_id));
                    }
                }
            }
        }

        // Append new page refs to the target's Kids array
        if let Some(pages_id) = target_pages_id {
            if let Ok(pages_obj) = target.get_object_mut(pages_id) {
                if let Ok(pages_dict) = pages_obj.as_dict_mut() {
                    if let Ok(kids) = pages_dict.get_mut(b"Kids") {
                        if let Ok(kids_arr) = kids.as_array_mut() {
                            for new_id in &new_page_refs {
                                kids_arr.push(Object::Reference(*new_id));
                            }
                        }
                    }
                }
            }
        }

        // Update page count separately (avoids simultaneous mutable+immutable borrow)
        let new_count = target.get_pages().len() as i64;
        if let Some(pages_id) = target_pages_id {
            if let Ok(pages_obj) = target.get_object_mut(pages_id) {
                if let Ok(pages_dict) = pages_obj.as_dict_mut() {
                    pages_dict.set("Count", Object::Integer(new_count));
                }
            }
        }

        target.max_id = target
            .objects
            .keys()
            .map(|id| id.0)
            .max()
            .unwrap_or(target.max_id);

        // Merge AcroForm fields from the other document
        if let Some(other_catalog_id) = other_root_id {
            let _ = merge_acroforms(&mut target, other_catalog_id, doc_idx + 2);
        }
    }

    target.renumber_objects();
    target.adjust_zero_pages();

    Document::from_lopdf(target)
}

/// Merge AcroForm fields from a copied document into the target.
///
/// Finds the other document's AcroForm (via its remapped catalog), prefixes
/// field names with "doc{N}_", and appends them to the target's AcroForm.
fn merge_acroforms(
    target: &mut lopdf::Document,
    other_catalog_id: lopdf::ObjectId,
    doc_index: usize,
) -> Result<()> {
    // Get the other doc's catalog
    let other_catalog = match target.get_object(other_catalog_id) {
        Ok(obj) => match obj.as_dict() {
            Ok(d) => d.clone(),
            Err(_) => return Ok(()),
        },
        Err(_) => return Ok(()),
    };

    // Get the other doc's AcroForm
    let other_acroform_obj = match other_catalog.get(b"AcroForm") {
        Ok(obj) => obj.clone(),
        Err(_) => return Ok(()), // No form in other doc
    };

    let other_af_dict = match &other_acroform_obj {
        Object::Dictionary(d) => d.clone(),
        Object::Reference(id) => match target.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => return Ok(()),
            },
            Err(_) => return Ok(()),
        },
        _ => return Ok(()),
    };

    // Extract the field refs from the other AcroForm
    let other_fields = match other_af_dict.get(b"Fields") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match target.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };

    if other_fields.is_empty() {
        return Ok(());
    }

    // Prefix top-level field names with "doc{N}_"
    let prefix = format!("doc{}_", doc_index);
    for field_ref in &other_fields {
        if let Ok(field_id) = field_ref.as_reference() {
            if let Ok(field_obj) = target.get_object_mut(field_id) {
                if let Ok(field_dict) = field_obj.as_dict_mut() {
                    if let Ok(Object::String(bytes, fmt)) = field_dict.get(b"T") {
                        let old_name = String::from_utf8_lossy(bytes).to_string();
                        let new_name = format!("{}{}", prefix, old_name);
                        let fmt = *fmt;
                        field_dict.set("T", Object::String(new_name.into_bytes(), fmt));
                    }
                }
            }
        }
    }

    // Ensure target has an AcroForm
    let target_root_id = target
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Structure("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Structure("/Root not ref".to_string()))?;

    let target_af_ref = {
        let target_root = target
            .get_object(target_root_id)
            .map_err(|e| PdfError::Structure(format!("Root get: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Structure("/Root not dict".to_string()))?;

        target_root.get(b"AcroForm").ok().and_then(|o| match o {
            Object::Reference(id) => Some(*id),
            _ => None,
        })
    };

    if let Some(af_id) = target_af_ref {
        // Append fields to existing AcroForm
        let af_obj = target
            .get_object_mut(af_id)
            .map_err(|e| PdfError::Structure(format!("AcroForm get: {}", e)))?;
        let af_dict = af_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Structure("AcroForm not dict".to_string()))?;

        match af_dict.get_mut(b"Fields") {
            Ok(Object::Array(fields)) => {
                fields.extend(other_fields);
            }
            _ => {
                af_dict.set("Fields", Object::Array(other_fields));
            }
        }

        // Merge /DR (default resources) — merge font dicts additively
        if let Ok(other_dr) = other_af_dict.get(b"DR") {
            merge_dr_into(target, af_id, other_dr.clone())?;
        }

        // Merge /SigFlags — OR the bits together
        if let Ok(Object::Integer(other_flags)) = other_af_dict.get(b"SigFlags") {
            let af_obj = target
                .get_object_mut(af_id)
                .map_err(|e| PdfError::Structure(format!("AcroForm get: {}", e)))?;
            let af_dict = af_obj
                .as_dict_mut()
                .map_err(|_| PdfError::Structure("AcroForm not dict".to_string()))?;
            let current = af_dict
                .get(b"SigFlags")
                .ok()
                .and_then(|o| match o {
                    Object::Integer(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0);
            af_dict.set("SigFlags", Object::Integer(current | other_flags));
        }
    } else {
        // Target has no AcroForm — create one with the other doc's fields
        let acroform = dictionary! {
            "Fields" => Object::Array(other_fields),
        };
        let af_id = target.new_object_id();
        target.objects.insert(af_id, Object::Dictionary(acroform));

        let root_obj = target
            .get_object_mut(target_root_id)
            .map_err(|e| PdfError::Structure(format!("Root get: {}", e)))?;
        let root_dict = root_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Structure("/Root not dict".to_string()))?;
        root_dict.set("AcroForm", Object::Reference(af_id));

        // Copy DR and SigFlags if present
        if let Ok(other_dr) = other_af_dict.get(b"DR") {
            merge_dr_into(target, af_id, other_dr.clone())?;
        }
        if let Ok(Object::Integer(flags)) = other_af_dict.get(b"SigFlags") {
            let af_obj = target
                .get_object_mut(af_id)
                .map_err(|e| PdfError::Structure(format!("AcroForm get: {}", e)))?;
            let af_dict = af_obj
                .as_dict_mut()
                .map_err(|_| PdfError::Structure("AcroForm not dict".to_string()))?;
            af_dict.set("SigFlags", Object::Integer(*flags));
        }
    }

    // Remove /AcroForm from the other doc's copied catalog to avoid orphaned refs
    if let Ok(catalog_obj) = target.get_object_mut(other_catalog_id) {
        if let Ok(catalog_dict) = catalog_obj.as_dict_mut() {
            catalog_dict.remove(b"AcroForm");
        }
    }

    Ok(())
}

/// Merge /DR (default resources) from another AcroForm into the target's AcroForm.
fn merge_dr_into(
    target: &mut lopdf::Document,
    af_id: lopdf::ObjectId,
    other_dr: Object,
) -> Result<()> {
    let other_dr_dict = match &other_dr {
        Object::Dictionary(d) => d.clone(),
        Object::Reference(id) => match target.get_object(*id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => return Ok(()),
            },
            Err(_) => return Ok(()),
        },
        _ => return Ok(()),
    };

    let af_obj = target
        .get_object_mut(af_id)
        .map_err(|e| PdfError::Structure(format!("AcroForm get: {}", e)))?;
    let af_dict = af_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Structure("AcroForm not dict".to_string()))?;

    if let Ok(Object::Dictionary(target_dr)) = af_dict.get_mut(b"DR") {
        // Merge font dict additively
        if let Ok(Object::Dictionary(other_fonts)) = other_dr_dict.get(b"Font") {
            if let Ok(Object::Dictionary(target_fonts)) = target_dr.get_mut(b"Font") {
                for (key, val) in other_fonts.iter() {
                    if target_fonts.get(key).is_err() {
                        target_fonts.set(std::str::from_utf8(key).unwrap_or(""), val.clone());
                    }
                }
            } else {
                target_dr.set("Font", Object::Dictionary(other_fonts.clone()));
            }
        }
    } else {
        af_dict.set("DR", Object::Dictionary(other_dr_dict));
    }

    Ok(())
}

/// Merge PDF files from paths.
pub fn merge_files<P: AsRef<Path>>(paths: &[P], options: &MergeOptions) -> Result<Document> {
    let docs: Result<Vec<Document>> = paths.iter().map(Document::open).collect();
    merge(docs?, options)
}
