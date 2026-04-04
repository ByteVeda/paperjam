use paperjam_model::structure::ContentBlock;

use crate::document::EpubDocument;
use crate::error::EpubError;

impl EpubDocument {
    pub fn extract_structure(&self) -> Result<Vec<ContentBlock>, EpubError> {
        let mut all_blocks = Vec::new();
        for ch in &self.chapters {
            let page = (ch.index + 1) as u32;
            let blocks = paperjam_html::structure::extract_structure_from_html_with_page(
                ch.html.dom(),
                page,
            );
            all_blocks.extend(blocks);
        }
        Ok(all_blocks)
    }
}
