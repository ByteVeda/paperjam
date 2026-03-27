use lopdf::{dictionary, Object, ObjectId};

use crate::document::Document;
use crate::error::{PdfError, Result};

/// Insert blank pages into a document at specified positions.
///
/// `positions` is a list of `(after_page, width, height)` tuples:
/// - `after_page = 0` inserts at the very beginning
/// - `after_page = N` inserts after page N (1-indexed)
/// - `width` and `height` are in PDF points (72 points = 1 inch)
///
/// Returns a new document with the blank pages inserted.
pub fn insert_blank_pages(doc: &Document, positions: &[(u32, f64, f64)]) -> Result<Document> {
    let mut new_doc = doc.inner().clone();
    let page_count = doc.page_count() as u32;

    // Validate positions
    for &(after_page, width, height) in positions {
        if after_page > page_count {
            return Err(PdfError::PageOutOfRange {
                page: after_page as usize,
                total: page_count as usize,
            });
        }
        if width <= 0.0 || height <= 0.0 {
            return Err(PdfError::Structure(
                "Page width and height must be positive".into(),
            ));
        }
    }

    // Find the Pages root object ID
    let pages_id = new_doc
        .catalog()
        .ok()
        .and_then(|cat| cat.get(b"Pages").ok())
        .and_then(|p| p.as_reference().ok())
        .ok_or_else(|| PdfError::Structure("Cannot find Pages root in catalog".into()))?;

    // Sort positions in reverse order so insertions don't shift indices
    let mut sorted_positions = positions.to_vec();
    sorted_positions.sort_by(|a, b| b.0.cmp(&a.0));

    for (after_page, width, height) in sorted_positions {
        // Create an empty content stream
        let content_stream = lopdf::Stream::new(dictionary! {}, vec![]);
        let content_id = new_doc.add_object(Object::Stream(content_stream));

        // Create the page object
        let page_dict = dictionary! {
            "Type" => Object::Name(b"Page".to_vec()),
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Real(width as f32),
                Object::Real(height as f32),
            ]),
            "Contents" => Object::Reference(content_id),
            "Resources" => Object::Dictionary(lopdf::Dictionary::new())
        };
        let page_id = new_doc.add_object(Object::Dictionary(page_dict));

        // Insert the new page ref into the Kids array
        insert_page_into_kids(&mut new_doc, pages_id, page_id, after_page)?;
    }

    new_doc.renumber_objects();
    new_doc.adjust_zero_pages();

    Document::from_lopdf(new_doc)
}

/// Insert a page reference into the /Kids array of the /Pages root and increment /Count.
fn insert_page_into_kids(
    doc: &mut lopdf::Document,
    pages_id: ObjectId,
    new_page_id: ObjectId,
    after_page: u32,
) -> Result<()> {
    let pages_obj = doc.get_object_mut(pages_id).map_err(PdfError::Lopdf)?;
    let pages_dict = pages_obj.as_dict_mut().map_err(PdfError::Lopdf)?;

    let kids = pages_dict.get_mut(b"Kids").map_err(PdfError::Lopdf)?;

    if let Object::Array(ref mut kids_arr) = kids {
        let insert_idx = after_page as usize;
        kids_arr.insert(insert_idx, Object::Reference(new_page_id));
    }

    // Update Count
    let count = pages_dict
        .get(b"Count")
        .ok()
        .and_then(|c| c.as_i64().ok())
        .unwrap_or(0);
    pages_dict.set("Count", Object::Integer(count + 1));

    Ok(())
}
