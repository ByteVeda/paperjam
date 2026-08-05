/// A single bookmark entry from the document's outline/TOC.
#[derive(Debug, Clone)]
pub struct BookmarkItem {
    pub title: String,
    pub page: usize,
    pub level: usize,
}

/// A bookmark specification for creating outlines.
#[derive(Debug, Clone)]
pub struct BookmarkSpec {
    pub title: String,
    pub page: u32,
    pub children: Vec<BookmarkSpec>,
}
