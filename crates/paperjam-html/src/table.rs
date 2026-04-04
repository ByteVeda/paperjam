use scraper::{ElementRef, Selector};

use paperjam_model::table::{Cell, Row, Table};

use crate::document::HtmlDocument;
use crate::error::HtmlError;

/// Extract all `<table>` elements from a parsed HTML DOM.
pub fn extract_tables_from_html(dom: &scraper::Html) -> Vec<Table> {
    let table_sel = match Selector::parse("table") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    dom.select(&table_sel)
        .filter_map(|table_el| parse_table(table_el))
        .collect()
}

fn parse_table(table_el: ElementRef) -> Option<Table> {
    let tr_sel = Selector::parse("tr").ok()?;
    let td_sel = Selector::parse("td, th").ok()?;

    let mut rows = Vec::new();
    let mut max_cols: usize = 0;

    for (row_idx, tr) in table_el.select(&tr_sel).enumerate() {
        let mut cells = Vec::new();
        for (col_idx, td) in tr.select(&td_sel).enumerate() {
            let text = td.text().collect::<String>().trim().to_string();

            let col_span = td
                .value()
                .attr("colspan")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            let row_span = td
                .value()
                .attr("rowspan")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);

            cells.push(Cell {
                text,
                bbox: (
                    col_idx as f64,
                    row_idx as f64,
                    col_idx as f64,
                    row_idx as f64,
                ),
                col_span,
                row_span,
            });
        }

        let logical_cols: usize = cells.iter().map(|c| c.col_span as usize).sum();
        if logical_cols > max_cols {
            max_cols = logical_cols;
        }

        rows.push(Row {
            cells,
            y_min: row_idx as f64,
            y_max: row_idx as f64,
        });
    }

    if rows.is_empty() {
        return None;
    }

    Some(Table {
        bbox: (0.0, 0.0, 0.0, 0.0),
        rows,
        col_count: max_cols,
        strategy: paperjam_model::table::TableStrategy::Auto,
    })
}

impl HtmlDocument {
    pub fn extract_tables(&self) -> Result<Vec<Table>, HtmlError> {
        Ok(extract_tables_from_html(&self.dom))
    }
}
