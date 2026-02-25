use std::collections::HashSet;

use lopdf::Object;

use crate::document::Document;
use crate::error::{PdfError, Result};
use crate::manipulation::utils::collect_refs;

/// Options controlling PDF optimization.
pub struct OptimizeOptions {
    /// Compress uncompressed streams using FlateDecode.
    pub compress_streams: bool,
    /// Remove objects not referenced by any page or the catalog.
    pub remove_unused_objects: bool,
    /// Remove duplicate stream objects.
    pub remove_duplicates: bool,
    /// Strip document metadata (/Info and /Metadata).
    pub strip_metadata: bool,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            compress_streams: true,
            remove_unused_objects: true,
            remove_duplicates: true,
            strip_metadata: false,
        }
    }
}

/// Result statistics from optimization.
pub struct OptimizeResult {
    pub original_size: usize,
    pub optimized_size: usize,
    pub objects_removed: usize,
    pub streams_compressed: usize,
}

/// Optimize a PDF document by removing unused objects, compressing streams,
/// removing duplicates, and optionally stripping metadata.
///
/// Returns a new optimized Document and statistics about what changed.
pub fn optimize(doc: &Document, options: &OptimizeOptions) -> Result<(Document, OptimizeResult)> {
    // Measure original size
    let original_size = {
        let mut buf = Vec::new();
        let mut clone = doc.inner().clone();
        clone
            .save_to(&mut buf)
            .map_err(|e| PdfError::Structure(format!("Failed to measure original size: {}", e)))?;
        buf.len()
    };

    let mut inner = doc.inner().clone();
    let mut objects_removed = 0usize;
    let mut streams_compressed = 0usize;

    // 1. Remove unused objects
    if options.remove_unused_objects {
        let removed = remove_unused(&mut inner);
        objects_removed += removed;
    }

    // 2. Remove duplicate stream objects
    if options.remove_duplicates {
        let removed = remove_duplicates(&mut inner);
        objects_removed += removed;
    }

    // 3. Compress uncompressed streams
    if options.compress_streams {
        streams_compressed = compress_streams(&mut inner);
    }

    // 4. Strip metadata
    if options.strip_metadata {
        strip_metadata(&mut inner);
    }

    inner.renumber_objects();
    inner.adjust_zero_pages();

    // Measure optimized size
    let optimized_size = {
        let mut buf = Vec::new();
        let mut measure_clone = inner.clone();
        measure_clone.save_to(&mut buf).map_err(|e| {
            PdfError::Structure(format!("Failed to measure optimized size: {}", e))
        })?;
        buf.len()
    };

    let result_doc = Document::from_lopdf(inner)?;

    Ok((
        result_doc,
        OptimizeResult {
            original_size,
            optimized_size,
            objects_removed,
            streams_compressed,
        },
    ))
}

/// Remove objects not reachable from the document's root catalog.
fn remove_unused(doc: &mut lopdf::Document) -> usize {
    let mut reachable = HashSet::new();

    // Start from the trailer's Root reference
    if let Ok(root_ref) = doc.trailer.get(b"Root") {
        if let Ok(id) = root_ref.as_reference() {
            collect_refs(doc, id, &mut reachable);
        }
    }

    // Also keep objects referenced from /Info
    if let Ok(info_ref) = doc.trailer.get(b"Info") {
        if let Ok(id) = info_ref.as_reference() {
            collect_refs(doc, id, &mut reachable);
        }
    }

    let all_ids: Vec<_> = doc.objects.keys().copied().collect();
    let mut removed = 0;
    for id in all_ids {
        if !reachable.contains(&id) {
            doc.objects.remove(&id);
            removed += 1;
        }
    }
    removed
}

/// Remove duplicate stream objects by comparing content and dictionary.
/// When duplicates are found, all references to removed objects are rewritten.
fn remove_duplicates(doc: &mut lopdf::Document) -> usize {
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    // Build a hash map of stream content -> first object ID with that content
    let mut seen: HashMap<u64, lopdf::ObjectId> = HashMap::new();
    let mut remap: HashMap<lopdf::ObjectId, lopdf::ObjectId> = HashMap::new();

    let ids: Vec<_> = doc.objects.keys().copied().collect();
    for id in &ids {
        if let Some(Object::Stream(stream)) = doc.objects.get(id) {
            // Hash the stream content + filter info
            let mut hasher = std::hash::DefaultHasher::new();
            stream.content.hash(&mut hasher);
            // Include the filter key in the hash to avoid merging streams with different filters
            if let Ok(filter) = stream.dict.get(b"Filter") {
                format!("{:?}", filter).hash(&mut hasher);
            }
            let hash = hasher.finish();

            if let Some(&canonical_id) = seen.get(&hash) {
                if canonical_id != *id {
                    // Verify actual content equality (hash collision check)
                    let canonical_stream = match doc.objects.get(&canonical_id) {
                        Some(Object::Stream(s)) => s,
                        _ => continue,
                    };
                    if canonical_stream.content == stream.content {
                        remap.insert(*id, canonical_id);
                    }
                }
            } else {
                seen.insert(hash, *id);
            }
        }
    }

    if remap.is_empty() {
        return 0;
    }

    let removed = remap.len();

    // Remove duplicate objects
    for id in remap.keys() {
        doc.objects.remove(id);
    }

    // Rewrite all references
    let all_ids: Vec<_> = doc.objects.keys().copied().collect();
    for id in all_ids {
        if let Some(obj) = doc.objects.get_mut(&id) {
            rewrite_refs(obj, &remap);
        }
    }

    // Also rewrite trailer references
    let trailer_keys: Vec<Vec<u8>> = doc.trailer.iter().map(|(k, _)| k.to_vec()).collect();
    for key in trailer_keys {
        if let Ok(val) = doc.trailer.get_mut(&key) {
            rewrite_refs(val, &remap);
        }
    }

    removed
}

/// Recursively rewrite object references according to a remap table.
fn rewrite_refs(obj: &mut Object, remap: &std::collections::HashMap<lopdf::ObjectId, lopdf::ObjectId>) {
    match obj {
        Object::Reference(id) => {
            if let Some(&new_id) = remap.get(id) {
                *id = new_id;
            }
        }
        Object::Array(arr) => {
            for item in arr.iter_mut() {
                rewrite_refs(item, remap);
            }
        }
        Object::Dictionary(dict) => {
            for (_, item) in dict.iter_mut() {
                rewrite_refs(item, remap);
            }
        }
        Object::Stream(stream) => {
            for (_, item) in stream.dict.iter_mut() {
                rewrite_refs(item, remap);
            }
        }
        _ => {}
    }
}

/// Compress uncompressed streams using FlateDecode.
fn compress_streams(doc: &mut lopdf::Document) -> usize {
    let mut compressed = 0;
    let ids: Vec<_> = doc.objects.keys().copied().collect();

    for id in ids {
        let should_compress = if let Some(Object::Stream(stream)) = doc.objects.get(&id) {
            // Only compress if no filter is already applied
            stream.dict.get(b"Filter").is_err()
        } else {
            false
        };

        if should_compress {
            if let Some(Object::Stream(stream)) = doc.objects.get_mut(&id) {
                if stream.compress().is_ok() {
                    compressed += 1;
                }
            }
        }
    }

    compressed
}

/// Strip document metadata from the PDF.
fn strip_metadata(doc: &mut lopdf::Document) {
    // Remove /Info reference from trailer
    if let Ok(info_ref) = doc.trailer.get(b"Info") {
        if let Ok(id) = info_ref.as_reference() {
            doc.objects.remove(&id);
        }
    }
    doc.trailer.remove(b"Info");

    // Remove /Metadata from the catalog
    // First, find the metadata object ID to remove
    let meta_id_to_remove = if let Ok(root_ref) = doc.trailer.get(b"Root") {
        if let Ok(root_id) = root_ref.as_reference() {
            if let Some(obj) = doc.objects.get(&root_id) {
                if let Ok(dict) = obj.as_dict() {
                    dict.get(b"Metadata")
                        .ok()
                        .and_then(|m| m.as_reference().ok())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Remove the metadata object
    if let Some(meta_id) = meta_id_to_remove {
        doc.objects.remove(&meta_id);
    }

    // Remove /Metadata key from the catalog dict
    if let Ok(root_ref) = doc.trailer.get(b"Root") {
        if let Ok(root_id) = root_ref.as_reference() {
            if let Some(obj) = doc.objects.get_mut(&root_id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    dict.remove(b"Metadata");
                }
            }
        }
    }
}
