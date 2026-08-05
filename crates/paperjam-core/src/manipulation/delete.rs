use crate::document::Document;
use crate::error::{PdfError, Result};

/// Delete specific pages from a document, returning a new document.
///
/// `pages_to_delete` is a list of 1-indexed page numbers to remove.
/// At least one page must remain after deletion.
pub fn delete_pages(doc: &Document, pages_to_delete: &[u32]) -> Result<Document> {
    let page_count = doc.page_count() as u32;

    // Validate all page numbers are in range
    for &p in pages_to_delete {
        if p < 1 || p > page_count {
            return Err(PdfError::PageOutOfRange {
                page: p as usize,
                total: page_count as usize,
            });
        }
    }

    // Build the complement: all pages NOT in the delete set
    let delete_set: std::collections::HashSet<u32> = pages_to_delete.iter().copied().collect();
    let new_order: Vec<u32> = (1..=page_count)
        .filter(|p| !delete_set.contains(p))
        .collect();

    if new_order.is_empty() {
        return Err(PdfError::Structure(
            "Cannot delete all pages; at least one page must remain".into(),
        ));
    }

    super::reorder::reorder_pages(doc, &new_order)
}
