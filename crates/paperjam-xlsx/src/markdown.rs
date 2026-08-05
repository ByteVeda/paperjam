use crate::document::XlsxDocument;

/// Convert the workbook to a Markdown string with pipe tables.
pub fn sheets_to_markdown(doc: &XlsxDocument) -> String {
    let mut out = String::new();

    for (i, sheet) in doc.sheets.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n\n");
        }

        out.push_str(&format!("# {}\n\n", sheet.name));

        if sheet.rows.is_empty() {
            continue;
        }

        let col_count = sheet.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if col_count == 0 {
            continue;
        }

        for (row_idx, row) in sheet.rows.iter().enumerate() {
            out.push('|');
            for col_idx in 0..col_count {
                let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                // Escape pipe characters inside cell text.
                let escaped = cell.replace('|', "\\|");
                out.push_str(&format!(" {escaped} |"));
            }
            out.push('\n');

            // Insert separator after the first row (header).
            if row_idx == 0 {
                out.push('|');
                for _ in 0..col_count {
                    out.push_str(" --- |");
                }
                out.push('\n');
            }
        }

        out.push('\n');
    }

    out
}
