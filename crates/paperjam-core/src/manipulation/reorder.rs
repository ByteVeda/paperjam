use std::collections::HashSet;

use lopdf::Object;

use crate::document::Document;
use crate::error::{PdfError, Result};

/// Reorder pages in a document. `new_order` is a list of 1-indexed page numbers
/// in the desired order. Can include a subset (to also drop pages) or repeat
/// pages (to duplicate them).
pub fn reorder_pages(doc: &Document, new_order: &[u32]) -> Result<Document> {
    let source = doc.inner();
    let page_map = source.get_pages();

    // Validate all page numbers exist
    for &p in new_order {
        if !page_map.contains_key(&p) {
            return Err(PdfError::PageOutOfRange {
                page: p as usize,
                total: page_map.len(),
            });
        }
    }

    let mut new_doc = source.clone();

    // Build the new Kids array from the requested order
    let new_kids: Vec<Object> = new_order
        .iter()
        .filter_map(|&p| page_map.get(&p).map(|id| Object::Reference(*id)))
        .collect();

    // Find the Pages root
    let pages_id = new_doc
        .catalog()
        .ok()
        .and_then(|cat| cat.get(b"Pages").ok())
        .and_then(|p| p.as_reference().ok());

    if let Some(pages_id) = pages_id {
        if let Ok(pages_obj) = new_doc.get_object_mut(pages_id) {
            if let Ok(pages_dict) = pages_obj.as_dict_mut() {
                pages_dict.set("Kids", Object::Array(new_kids));
                pages_dict.set("Count", Object::Integer(new_order.len() as i64));
            }
        }
    }

    // Remove objects for pages not in the new order
    let kept_page_ids: HashSet<lopdf::ObjectId> = new_order
        .iter()
        .filter_map(|&p| page_map.get(&p).copied())
        .collect();

    let removed_page_ids: Vec<lopdf::ObjectId> = page_map
        .values()
        .filter(|id| !kept_page_ids.contains(id))
        .copied()
        .collect();

    for id in removed_page_ids {
        new_doc.objects.remove(&id);
    }

    new_doc.renumber_objects();
    new_doc.adjust_zero_pages();

    Document::from_lopdf(new_doc)
}
