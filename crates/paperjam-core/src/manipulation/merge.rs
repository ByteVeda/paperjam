use crate::document::Document;
use crate::error::Result;
use std::path::Path;

use lopdf::Object;

#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    pub deduplicate_resources: bool,
}

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
pub fn merge(documents: Vec<Document>, _options: &MergeOptions) -> Result<Document> {
    if documents.is_empty() {
        return Err(crate::error::PdfError::Structure(
            "Cannot merge zero documents".into(),
        ));
    }

    let mut inners: Vec<lopdf::Document> = documents.into_iter().map(|d| d.into_inner()).collect();
    let mut target = inners.remove(0);

    for other in inners {
        // Manually merge: renumber objects in `other` to avoid collisions,
        // then copy objects and append pages.
        let max_id = target.max_id;
        let other_pages = other.get_pages();

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
    }

    target.renumber_objects();
    target.adjust_zero_pages();

    Document::from_lopdf(target)
}

/// Merge PDF files from paths.
pub fn merge_files<P: AsRef<Path>>(paths: &[P], options: &MergeOptions) -> Result<Document> {
    let docs: Result<Vec<Document>> = paths.iter().map(|p| Document::open(p)).collect();
    merge(docs?, options)
}
