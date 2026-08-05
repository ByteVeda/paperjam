use crate::document::SlideData;
use paperjam_model::table::Table;

/// Collect all tables from every slide.
pub fn extract_all_tables(slides: &[SlideData]) -> Vec<Table> {
    slides.iter().flat_map(|s| s.tables.clone()).collect()
}
