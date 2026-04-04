use paperjam_model::bookmarks::BookmarkItem;
use paperjam_model::image::ImageInfo;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;

/// Format-agnostic document representation used during conversion.
#[derive(Debug, Clone)]
pub struct IntermediateDoc {
    pub metadata: Metadata,
    pub blocks: Vec<ContentBlock>,
    pub tables: Vec<Table>,
    pub images: Vec<ImageInfo>,
    pub bookmarks: Vec<BookmarkItem>,
}
