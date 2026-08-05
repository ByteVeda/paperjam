use paperjam_model::table::Table;

use crate::document::EpubDocument;
use crate::error::EpubError;

impl EpubDocument {
    pub fn extract_tables(&self) -> Result<Vec<Table>, EpubError> {
        let mut all_tables = Vec::new();
        for ch in &self.chapters {
            let tables = paperjam_html::table::extract_tables_from_html(ch.html.dom());
            all_tables.extend(tables);
        }
        Ok(all_tables)
    }
}
