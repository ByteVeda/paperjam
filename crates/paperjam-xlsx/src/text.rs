use paperjam_model::text::{TextLine, TextSpan};

use crate::document::XlsxDocument;

/// Extract all text from the workbook as a single string.
///
/// Cells within a row are tab-separated, rows are newline-separated,
/// and sheets are delimited by a header line.
pub fn extract_text(doc: &XlsxDocument) -> String {
    let mut out = String::new();

    for (i, sheet) in doc.sheets.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("=== Sheet: {} ===\n", sheet.name));

        for row in &sheet.rows {
            let line: String = row.join("\t");
            out.push_str(&line);
            out.push('\n');
        }
    }

    out
}

/// Extract text as positioned [`TextLine`]s.
///
/// Each row in each sheet produces one `TextLine` whose single span
/// contains the tab-joined cell values. Coordinates are synthetic
/// (row index as Y, 0 as X) since XLSX cells have no spatial position.
pub fn extract_text_lines(doc: &XlsxDocument) -> Vec<TextLine> {
    let mut lines = Vec::new();

    for (sheet_idx, sheet) in doc.sheets.iter().enumerate() {
        for (row_idx, row) in sheet.rows.iter().enumerate() {
            let text = row.join("\t");
            if text.is_empty() {
                continue;
            }
            let y = (sheet_idx * 10000 + row_idx) as f64;
            let span = TextSpan {
                text,
                x: 0.0,
                y,
                width: 0.0,
                font_size: 12.0,
                font_name: String::new(),
            };
            lines.push(TextLine {
                spans: vec![span],
                bbox: (0.0, y, 0.0, y + 12.0),
            });
        }
    }

    lines
}
