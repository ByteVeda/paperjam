use std::collections::HashSet;

use lopdf::{dictionary, Object, ObjectId};

use crate::document::Document;
use crate::error::Result;

/// Recursively collect all ObjectIds referenced by a given object.
fn collect_refs(doc: &lopdf::Document, id: ObjectId, visited: &mut HashSet<ObjectId>) {
    if !visited.insert(id) {
        return;
    }
    if let Ok(obj) = doc.get_object(id) {
        collect_refs_from_object(doc, obj, visited);
    }
}

/// Walk an Object's structure collecting referenced ObjectIds.
fn collect_refs_from_object(doc: &lopdf::Document, obj: &Object, visited: &mut HashSet<ObjectId>) {
    match obj {
        Object::Reference(id) => {
            collect_refs(doc, *id, visited);
        }
        Object::Array(arr) => {
            for item in arr {
                collect_refs_from_object(doc, item, visited);
            }
        }
        Object::Dictionary(dict) => {
            for (key, item) in dict.iter() {
                // Skip Parent references to avoid walking up the page tree
                if key == b"Parent" {
                    continue;
                }
                collect_refs_from_object(doc, item, visited);
            }
        }
        Object::Stream(stream) => {
            for (key, item) in stream.dict.iter() {
                if key == b"Parent" {
                    continue;
                }
                collect_refs_from_object(doc, item, visited);
            }
        }
        _ => {}
    }
}

/// Split a document into multiple documents by page ranges.
/// Each range is (start, end) 1-indexed, inclusive.
///
/// Uses an efficient "extract wanted pages" approach: builds a new document
/// containing only the selected pages and their dependencies, instead of
/// cloning the entire document and deleting unwanted pages.
pub fn split(doc: &Document, ranges: &[(u32, u32)]) -> Result<Vec<Document>> {
    let source = doc.inner();
    let page_map = source.get_pages();
    let mut results = Vec::new();

    for &(start, end) in ranges {
        // Collect the ObjectIds of the wanted pages
        let wanted_page_ids: Vec<ObjectId> = (start..=end)
            .filter_map(|p| page_map.get(&p).copied())
            .collect();

        // Collect all objects referenced by the wanted pages
        let mut needed_ids = HashSet::new();
        for &page_id in &wanted_page_ids {
            collect_refs(source, page_id, &mut needed_ids);
        }

        // Build a new lopdf document
        let mut new_doc = lopdf::Document::with_version(source.version.clone());

        // Copy all needed objects
        for &id in &needed_ids {
            if let Some(obj) = source.objects.get(&id) {
                new_doc.objects.insert(id, obj.clone());
            }
        }

        // Create a new Pages root node
        let pages_root_id = new_doc.new_object_id();
        let kids: Vec<Object> = wanted_page_ids
            .iter()
            .map(|id| Object::Reference(*id))
            .collect();
        let pages_dict = dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => Object::Integer(wanted_page_ids.len() as i64),
        };
        new_doc.objects.insert(pages_root_id, Object::Dictionary(pages_dict));

        // Update Parent references in each page to point to the new Pages root
        for &page_id in &wanted_page_ids {
            if let Some(obj) = new_doc.objects.get_mut(&page_id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.set("Parent", Object::Reference(pages_root_id));
                }
            }
        }

        // Create a new Catalog
        let catalog_id = new_doc.new_object_id();
        let catalog = dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_root_id),
        };
        new_doc.objects.insert(catalog_id, Object::Dictionary(catalog));

        // Set up the trailer
        new_doc.trailer.set("Root", Object::Reference(catalog_id));
        new_doc.max_id = new_doc
            .objects
            .keys()
            .map(|id| id.0)
            .max()
            .unwrap_or(0);

        new_doc.renumber_objects();
        new_doc.adjust_zero_pages();

        results.push(Document::from_lopdf(new_doc)?);
    }

    Ok(results)
}

/// Split into individual pages.
pub fn split_pages(doc: &Document) -> Result<Vec<Document>> {
    let count = doc.page_count() as u32;
    let ranges: Vec<(u32, u32)> = (1..=count).map(|i| (i, i)).collect();
    split(doc, &ranges)
}
