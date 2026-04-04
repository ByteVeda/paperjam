use std::collections::BTreeMap;

use lopdf::{dictionary, Object, ObjectId};

use crate::document::Document;
use crate::error::{PdfError, Result};

pub use paperjam_model::bookmarks::*;

/// Extract the document's bookmark/outline tree as a flat list with levels.
pub fn extract_bookmarks(doc: &lopdf::Document) -> Result<Vec<BookmarkItem>> {
    match doc.get_toc() {
        Ok(toc) => Ok(toc
            .toc
            .into_iter()
            .map(|entry| BookmarkItem {
                title: entry.title,
                page: entry.page,
                level: entry.level,
            })
            .collect()),
        Err(lopdf::Error::NoOutlines) => Ok(Vec::new()),
        Err(e) => Err(crate::error::PdfError::Lopdf(e)),
    }
}

/// Replace the document's bookmarks/outlines with the given tree.
///
/// If `bookmarks` is empty, any existing outlines are removed.
pub fn set_bookmarks(doc: &Document, bookmarks: &[BookmarkSpec]) -> Result<Document> {
    let mut new_doc = doc.inner().clone();
    let page_map = new_doc.get_pages();

    // Validate all page numbers
    for spec in bookmarks {
        validate_bookmark_pages(spec, &page_map)?;
    }

    // Remove existing outlines from catalog
    remove_existing_outlines(&mut new_doc);

    if bookmarks.is_empty() {
        return Document::from_lopdf(new_doc);
    }

    // Build the outline tree
    let outlines_id = new_doc.new_object_id();

    let child_ids = build_outline_children(&mut new_doc, bookmarks, outlines_id, &page_map)?;

    let total_count = count_all_items(bookmarks);

    let mut outlines_dict = dictionary! {
        "Type" => Object::Name(b"Outlines".to_vec()),
        "Count" => Object::Integer(total_count as i64)
    };

    if let Some(first) = child_ids.first() {
        outlines_dict.set("First", Object::Reference(*first));
    }
    if let Some(last) = child_ids.last() {
        outlines_dict.set("Last", Object::Reference(*last));
    }

    new_doc
        .objects
        .insert(outlines_id, Object::Dictionary(outlines_dict));

    // Add /Outlines to catalog
    if let Ok(catalog) = new_doc.catalog_mut() {
        catalog.set("Outlines", Object::Reference(outlines_id));
    }

    Document::from_lopdf(new_doc)
}

/// Validate that all page numbers in the bookmark tree are within range.
fn validate_bookmark_pages(spec: &BookmarkSpec, page_map: &BTreeMap<u32, ObjectId>) -> Result<()> {
    if !page_map.contains_key(&spec.page) {
        return Err(PdfError::PageOutOfRange {
            page: spec.page as usize,
            total: page_map.len(),
        });
    }
    for child in &spec.children {
        validate_bookmark_pages(child, page_map)?;
    }
    Ok(())
}

/// Remove the existing /Outlines tree from the catalog and delete its objects.
fn remove_existing_outlines(doc: &mut lopdf::Document) {
    let outlines_id = doc
        .catalog()
        .ok()
        .and_then(|cat| cat.get(b"Outlines").ok())
        .and_then(|o| o.as_reference().ok());

    if let Some(id) = outlines_id {
        // Remove the outlines tree objects recursively
        remove_outline_tree(doc, id);
        // Remove /Outlines from catalog
        if let Ok(catalog) = doc.catalog_mut() {
            catalog.remove(b"Outlines");
        }
    }
}

/// Recursively remove an outline node and all its children from the document.
fn remove_outline_tree(doc: &mut lopdf::Document, obj_id: ObjectId) {
    let children = if let Some(Object::Dictionary(dict)) = doc.objects.get(&obj_id) {
        let mut ids = Vec::new();
        if let Ok(Object::Reference(first)) = dict.get(b"First") {
            let mut current = Some(*first);
            while let Some(cur_id) = current {
                ids.push(cur_id);
                current = doc
                    .objects
                    .get(&cur_id)
                    .and_then(|o| o.as_dict().ok())
                    .and_then(|d| d.get(b"Next").ok())
                    .and_then(|o| o.as_reference().ok());
            }
        }
        ids
    } else {
        Vec::new()
    };

    for child_id in children {
        remove_outline_tree(doc, child_id);
    }
    doc.objects.remove(&obj_id);
}

/// Build outline item objects for a list of sibling bookmarks and link them.
fn build_outline_children(
    doc: &mut lopdf::Document,
    specs: &[BookmarkSpec],
    parent_id: ObjectId,
    page_map: &BTreeMap<u32, ObjectId>,
) -> Result<Vec<ObjectId>> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }

    // Pre-allocate IDs for all siblings
    let ids: Vec<ObjectId> = specs.iter().map(|_| doc.new_object_id()).collect();

    for (i, spec) in specs.iter().enumerate() {
        let item_id = ids[i];
        let page_obj_id = page_map[&spec.page];

        let mut item_dict = dictionary! {
            "Title" => Object::String(spec.title.as_bytes().to_vec(), lopdf::StringFormat::Literal),
            "Parent" => Object::Reference(parent_id),
            "Dest" => Object::Array(vec![
                Object::Reference(page_obj_id),
                Object::Name(b"Fit".to_vec()),
            ])
        };

        // Link siblings
        if i > 0 {
            item_dict.set("Prev", Object::Reference(ids[i - 1]));
        }
        if i + 1 < specs.len() {
            item_dict.set("Next", Object::Reference(ids[i + 1]));
        }

        // Recursively build children
        let child_ids = build_outline_children(doc, &spec.children, item_id, page_map)?;
        if !child_ids.is_empty() {
            item_dict.set("First", Object::Reference(child_ids[0]));
            item_dict.set("Last", Object::Reference(*child_ids.last().unwrap()));
            let child_count = count_all_items(&spec.children);
            item_dict.set("Count", Object::Integer(child_count as i64));
        }

        doc.objects.insert(item_id, Object::Dictionary(item_dict));
    }

    Ok(ids)
}

/// Count the total number of items in a bookmark tree (including nested children).
fn count_all_items(specs: &[BookmarkSpec]) -> usize {
    specs
        .iter()
        .fold(0, |acc, s| acc + 1 + count_all_items(&s.children))
}
