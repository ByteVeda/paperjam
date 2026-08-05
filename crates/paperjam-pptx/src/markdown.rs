use crate::document::SlideData;
use paperjam_model::table::Table;

/// Render all slides as a Markdown document.
pub fn slides_to_markdown(slides: &[SlideData]) -> String {
    let mut out = String::new();

    for (i, slide) in slides.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n\n");
        }

        // Slide heading
        let heading = match &slide.title {
            Some(t) => format!("## Slide {}: {}\n\n", slide.index, t),
            None => format!("## Slide {}\n\n", slide.index),
        };
        out.push_str(&heading);

        // Body text
        for block in &slide.text_blocks {
            if block.is_title {
                continue; // already in the heading
            }
            if block.is_bullet {
                let indent = "  ".repeat(block.level as usize);
                out.push_str(&format!("{indent}- {}\n", block.text));
            } else {
                out.push_str(&block.text);
                out.push_str("\n\n");
            }
        }

        // Tables
        for table in &slide.tables {
            out.push_str(&table_to_pipe_table(table));
            out.push('\n');
        }

        // Notes
        if let Some(ref notes) = slide.notes {
            if !notes.is_empty() {
                out.push_str(&format!("> **Notes:** {notes}\n\n"));
            }
        }
    }

    out
}

/// Render a `Table` as a Markdown pipe table.
fn table_to_pipe_table(table: &Table) -> String {
    if table.rows.is_empty() {
        return String::new();
    }

    let mut out = String::new();

    for (row_idx, row) in table.rows.iter().enumerate() {
        let cells: Vec<&str> = row.cells.iter().map(|c| c.text.as_str()).collect();
        out.push('|');
        for cell in &cells {
            out.push_str(&format!(" {} |", cell));
        }
        out.push('\n');

        // Separator after the first row (header)
        if row_idx == 0 {
            out.push('|');
            for _ in &cells {
                out.push_str(" --- |");
            }
            out.push('\n');
        }
    }

    out
}
