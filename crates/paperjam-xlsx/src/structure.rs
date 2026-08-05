use paperjam_model::structure::ContentBlock;

use crate::document::XlsxDocument;
use crate::table::extract_tables;

/// Extract document structure as a list of [`ContentBlock`]s.
///
/// Each sheet produces a heading followed by a table block.
pub fn extract_structure(doc: &XlsxDocument) -> Vec<ContentBlock> {
    let tables = extract_tables(doc);
    let mut blocks = Vec::new();

    for (i, sheet) in doc.sheets.iter().enumerate() {
        let page = (i + 1) as u32;

        blocks.push(ContentBlock::Heading {
            text: sheet.name.clone(),
            level: 1,
            bbox: (0.0, 0.0, 0.0, 0.0),
            page,
        });

        if let Some(table) = tables.get(i) {
            blocks.push(ContentBlock::Table {
                table: table.clone(),
                page,
            });
        }
    }

    blocks
}
