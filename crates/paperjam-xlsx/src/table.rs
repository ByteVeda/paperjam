use paperjam_model::table::{Cell, Row, Table, TableStrategy};

use crate::document::XlsxDocument;

/// Extract each sheet as a [`Table`].
pub fn extract_tables(doc: &XlsxDocument) -> Vec<Table> {
    doc.sheets
        .iter()
        .map(|sheet| {
            let col_count = sheet.rows.iter().map(|r| r.len()).max().unwrap_or(0);

            let rows: Vec<Row> = sheet
                .rows
                .iter()
                .enumerate()
                .map(|(row_idx, row)| {
                    let cells: Vec<Cell> = row
                        .iter()
                        .enumerate()
                        .map(|(col_idx, text)| Cell {
                            text: text.clone(),
                            bbox: (
                                col_idx as f64,
                                row_idx as f64,
                                col_idx as f64,
                                row_idx as f64,
                            ),
                            col_span: 1,
                            row_span: 1,
                        })
                        .collect();

                    Row {
                        cells,
                        y_min: row_idx as f64,
                        y_max: row_idx as f64,
                    }
                })
                .collect();

            Table {
                bbox: (0.0, 0.0, 0.0, 0.0),
                rows,
                col_count,
                strategy: TableStrategy::Auto,
            }
        })
        .collect()
}
