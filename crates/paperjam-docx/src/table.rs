use docx_rs::{DocumentChild, TableCellContent, TableChild, TableRowChild};
use paperjam_model::table::{Cell, Row, Table, TableStrategy};

use crate::document::DocxDocument;
use crate::error::DocxError;
use crate::text::extract_paragraph_text;

impl DocxDocument {
    /// Extract all tables from the document body.
    pub fn extract_tables(&self) -> Result<Vec<Table>, DocxError> {
        let mut tables = Vec::new();
        for child in &self.inner.document.children {
            if let DocumentChild::Table(t) = child {
                tables.push(convert_table(t));
            }
        }
        Ok(tables)
    }
}

fn convert_table(table: &docx_rs::Table) -> Table {
    let mut rows = Vec::new();
    let mut max_cols: usize = 0;

    for row_child in &table.rows {
        let TableChild::TableRow(row) = row_child;
        let mut cells = Vec::new();

        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            let text = cell_text(cell);
            cells.push(Cell {
                text,
                bbox: (0.0, 0.0, 0.0, 0.0),
                col_span: 1,
                row_span: 1,
            });
        }

        if cells.len() > max_cols {
            max_cols = cells.len();
        }

        rows.push(Row {
            cells,
            y_min: 0.0,
            y_max: 0.0,
        });
    }

    Table {
        bbox: (0.0, 0.0, 0.0, 0.0),
        rows,
        col_count: max_cols,
        strategy: TableStrategy::Auto,
    }
}

fn cell_text(cell: &docx_rs::TableCell) -> String {
    let mut parts = Vec::new();
    for content in &cell.children {
        match content {
            TableCellContent::Paragraph(p) => {
                let t = extract_paragraph_text(p);
                if !t.is_empty() {
                    parts.push(t);
                }
            }
            TableCellContent::Table(nested) => {
                parts.push(nested_table_text(nested));
            }
            _ => {}
        }
    }
    parts.join("\n")
}

fn nested_table_text(table: &docx_rs::Table) -> String {
    let mut parts = Vec::new();
    for row_child in &table.rows {
        let TableChild::TableRow(row) = row_child;
        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            let t = cell_text(cell);
            if !t.is_empty() {
                parts.push(t);
            }
        }
    }
    parts.join("\n")
}
