use paperjam_model::metadata::Metadata;

use crate::document::XlsxDocument;

/// Extract basic metadata from an [`XlsxDocument`].
///
/// For v1, we return page_count = number of sheets and leave
/// title/author as None (parsing `docProps/core.xml` is deferred).
pub fn extract_metadata(doc: &XlsxDocument) -> Metadata {
    Metadata {
        page_count: doc.sheets.len(),
        pdf_version: String::new(),
        ..Metadata::default()
    }
}
