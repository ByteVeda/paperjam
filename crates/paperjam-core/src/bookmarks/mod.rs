use crate::error::Result;

/// A single bookmark entry from the document's outline/TOC.
#[derive(Debug, Clone)]
pub struct BookmarkItem {
    pub title: String,
    pub page: usize,
    pub level: usize,
}

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
