use std::collections::HashSet;

use lopdf::{Object, ObjectId};

/// Recursively collect all ObjectIds referenced by a given object.
pub(crate) fn collect_refs(doc: &lopdf::Document, id: ObjectId, visited: &mut HashSet<ObjectId>) {
    if !visited.insert(id) {
        return;
    }
    if let Ok(obj) = doc.get_object(id) {
        collect_refs_from_object(doc, obj, visited);
    }
}

/// Walk an Object's structure collecting referenced ObjectIds.
pub(crate) fn collect_refs_from_object(
    doc: &lopdf::Document,
    obj: &Object,
    visited: &mut HashSet<ObjectId>,
) {
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
