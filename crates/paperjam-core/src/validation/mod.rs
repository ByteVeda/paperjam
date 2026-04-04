//! PDF validation: PDF/A conformance and PDF/UA accessibility checks.

mod pdf_a;
mod pdf_ua;

use lopdf::Object;

pub use pdf_a::validate_pdf_a;
pub use pdf_ua::validate_pdf_ua;

pub use paperjam_model::validation::*;

/// Get resolved resources dictionary from a page dict.
pub(crate) fn get_resources(
    doc: &lopdf::Document,
    page_dict: &lopdf::Dictionary,
) -> Option<lopdf::Dictionary> {
    match page_dict.get(b"Resources") {
        Ok(Object::Dictionary(d)) => Some(d.clone()),
        Ok(Object::Reference(id)) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .cloned(),
        _ => {
            if let Ok(parent_ref) = page_dict.get(b"Parent") {
                if let Ok(parent_id) = parent_ref.as_reference() {
                    if let Ok(parent_obj) = doc.get_object(parent_id) {
                        if let Ok(parent_dict) = parent_obj.as_dict() {
                            return get_resources(doc, parent_dict);
                        }
                    }
                }
            }
            None
        }
    }
}
