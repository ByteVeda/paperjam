use crate::document::Document;
use crate::error::Result;
use crate::page::Page;
use crate::structure::ContentBlock;
use crate::table::Table;

pub use paperjam_model::markdown::*;

/// Convert pre-extracted content blocks to a Markdown string.
pub fn blocks_to_markdown(blocks: &[ContentBlock], options: &MarkdownOptions) -> String {
    if blocks.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let mut current_page: Option<u32> = None;
    let mut in_list = false;

    for block in blocks {
        let block_page = block.page();

        // Handle page transitions
        if let Some(prev_page) = current_page {
            if block_page != prev_page {
                in_list = false;
                if !options.page_separator.is_empty() && !out.is_empty() {
                    ensure_blank_line(&mut out);
                    out.push_str(&options.page_separator);
                    out.push('\n');
                }
                if options.include_page_numbers {
                    ensure_blank_line(&mut out);
                    out.push_str(
                        &options
                            .page_number_format
                            .replace("{n}", &block_page.to_string()),
                    );
                    out.push('\n');
                }
            }
        } else if options.include_page_numbers {
            out.push_str(
                &options
                    .page_number_format
                    .replace("{n}", &block_page.to_string()),
            );
            out.push('\n');
        }

        current_page = Some(block_page);

        match block {
            ContentBlock::Heading { text, level, .. } => {
                in_list = false;
                let effective_level =
                    (*level as u16 + options.heading_offset as u16).clamp(1, 6) as u8;
                ensure_blank_line(&mut out);
                for _ in 0..effective_level {
                    out.push('#');
                }
                out.push(' ');
                out.push_str(text.trim());
                out.push('\n');
            }

            ContentBlock::Paragraph { text, .. } => {
                in_list = false;
                ensure_blank_line(&mut out);
                out.push_str(text.trim());
                out.push('\n');
            }

            ContentBlock::ListItem {
                text, indent_level, ..
            } => {
                if !in_list {
                    ensure_blank_line(&mut out);
                }
                in_list = true;

                let indent = "  ".repeat(*indent_level as usize);
                let cleaned = if options.normalize_list_markers {
                    strip_list_marker(text)
                } else {
                    text.trim().to_string()
                };

                out.push_str(&indent);
                out.push_str("- ");
                out.push_str(&cleaned);
                out.push('\n');
            }

            ContentBlock::Table { table, .. } => {
                in_list = false;
                ensure_blank_line(&mut out);
                if options.html_tables {
                    render_html_table(&mut out, table);
                } else {
                    render_pipe_table(&mut out, table, options.table_header_first_row);
                }
            }
        }
    }

    // Trim trailing whitespace, ensure single trailing newline
    let trimmed = out.trim_end();
    let mut result = trimmed.to_string();
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

/// Extract structure from a single page and convert to Markdown.
pub fn page_to_markdown(page: &Page, options: &MarkdownOptions) -> Result<String> {
    let blocks = crate::structure::extract_structure(page, &options.structure_options)?;
    Ok(blocks_to_markdown(&blocks, options))
}

/// Extract structure from all pages and convert to Markdown.
pub fn document_to_markdown(doc: &Document, options: &MarkdownOptions) -> Result<String> {
    let blocks = crate::structure::extract_document_structure(doc, &options.structure_options)?;
    Ok(blocks_to_markdown(&blocks, options))
}

// --- Helpers ---

/// Ensure the output ends with a blank line (for markdown spacing).
fn ensure_blank_line(out: &mut String) {
    if out.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
}

/// Strip leading list markers from text.
fn strip_list_marker(text: &str) -> String {
    let trimmed = text.trim_start();

    // Bullet markers
    for prefix in &[
        "- ",
        "* ",
        "\u{2022} ",
        "\u{2023} ",
        "\u{25E6} ",
        "\u{2043} ",
        "\u{2013} ",
        "\u{2014} ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.to_string();
        }
    }

    // Digit-based markers: "1." "12)" etc.
    let bytes = trimmed.as_bytes();
    if !bytes.is_empty() && bytes[0].is_ascii_digit() {
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len()
            && (bytes[i] == b'.' || bytes[i] == b')')
            && i + 1 < bytes.len()
            && bytes[i + 1] == b' '
        {
            return trimmed[i + 2..].to_string();
        }
    }

    // Letter-based markers: "a. " "b) "
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && (bytes[1] == b'.' || bytes[1] == b')')
        && bytes[2] == b' '
    {
        return trimmed[3..].to_string();
    }

    trimmed.to_string()
}

/// Escape pipe characters in cell text for markdown tables.
fn escape_pipe(text: &str) -> String {
    text.trim().replace('|', "\\|").replace('\n', " ")
}

/// Render a table as a markdown pipe table.
fn render_pipe_table(out: &mut String, table: &Table, header_first_row: bool) {
    if table.rows.is_empty() {
        return;
    }

    let col_count = table.col_count;

    // Compute column widths (minimum 3 for "---")
    let mut col_widths: Vec<usize> = vec![3; col_count];
    for row in &table.rows {
        for (j, cell) in row.cells.iter().enumerate() {
            if j < col_count {
                let cell_text = escape_pipe(&cell.text);
                col_widths[j] = col_widths[j].max(cell_text.len());
            }
        }
    }

    let rows_vec: Vec<Vec<String>> = table
        .rows
        .iter()
        .map(|row| {
            (0..col_count)
                .map(|j| {
                    row.cells
                        .get(j)
                        .map(|c| escape_pipe(&c.text))
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    if header_first_row && !rows_vec.is_empty() {
        render_pipe_row(out, &rows_vec[0], &col_widths);
        render_separator_row(out, &col_widths);
        for row in &rows_vec[1..] {
            render_pipe_row(out, row, &col_widths);
        }
    } else {
        // No header — synthesize an empty header row
        let empty_header: Vec<String> = vec![String::new(); col_count];
        render_pipe_row(out, &empty_header, &col_widths);
        render_separator_row(out, &col_widths);
        for row in &rows_vec {
            render_pipe_row(out, row, &col_widths);
        }
    }
}

fn render_pipe_row(out: &mut String, cells: &[String], widths: &[usize]) {
    out.push('|');
    for (j, cell) in cells.iter().enumerate() {
        let w = widths.get(j).copied().unwrap_or(3);
        out.push(' ');
        out.push_str(cell);
        for _ in cell.len()..w {
            out.push(' ');
        }
        out.push_str(" |");
    }
    out.push('\n');
}

fn render_separator_row(out: &mut String, widths: &[usize]) {
    out.push('|');
    for w in widths {
        out.push(' ');
        for _ in 0..*w {
            out.push('-');
        }
        out.push_str(" |");
    }
    out.push('\n');
}

/// Render a table as HTML.
fn render_html_table(out: &mut String, table: &Table) {
    out.push_str("<table>\n");

    for (i, row) in table.rows.iter().enumerate() {
        out.push_str("  <tr>\n");
        let tag = if i == 0 { "th" } else { "td" };
        for cell in &row.cells {
            out.push_str("    <");
            out.push_str(tag);
            if cell.col_span > 1 {
                out.push_str(&format!(" colspan=\"{}\"", cell.col_span));
            }
            if cell.row_span > 1 {
                out.push_str(&format!(" rowspan=\"{}\"", cell.row_span));
            }
            out.push('>');
            out.push_str(&html_escape(&cell.text));
            out.push_str("</");
            out.push_str(tag);
            out.push_str(">\n");
        }
        out.push_str("  </tr>\n");
    }

    out.push_str("</table>\n");
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
