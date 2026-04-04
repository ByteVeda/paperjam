use docx_rs::{DocumentChild, StructuredDataTagChild, TableChild, TableRowChild};
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::{Cell, Row, Table, TableStrategy};

use crate::document::DocxDocument;
use crate::error::DocxError;
use crate::text::extract_paragraph_text;

impl DocxDocument {
    /// Extract the document structure as a sequence of content blocks.
    pub fn extract_structure(&self) -> Result<Vec<ContentBlock>, DocxError> {
        let mut blocks = Vec::new();
        for child in &self.inner.document.children {
            collect_blocks(child, &mut blocks);
        }
        Ok(blocks)
    }
}

fn collect_blocks(child: &DocumentChild, out: &mut Vec<ContentBlock>) {
    match child {
        DocumentChild::Paragraph(p) => {
            let text = extract_paragraph_text(p);
            if text.is_empty() {
                return;
            }

            // Detect heading by paragraph style id
            if let Some(ref style) = p.property.style {
                if let Some(level) = heading_level_from_style(&style.val) {
                    out.push(ContentBlock::Heading {
                        text,
                        level,
                        bbox: (0.0, 0.0, 0.0, 0.0),
                        page: 1,
                    });
                    return;
                }
            }

            // Detect list item by numbering property
            if p.property.numbering_property.is_some() {
                let indent_level = p
                    .property
                    .numbering_property
                    .as_ref()
                    .and_then(|np| np.level.as_ref())
                    .map(|lvl| lvl.val as u8)
                    .unwrap_or(0);
                out.push(ContentBlock::ListItem {
                    text,
                    indent_level,
                    bbox: (0.0, 0.0, 0.0, 0.0),
                    page: 1,
                });
                return;
            }

            // Plain paragraph
            out.push(ContentBlock::Paragraph {
                text,
                bbox: (0.0, 0.0, 0.0, 0.0),
                page: 1,
            });
        }
        DocumentChild::Table(t) => {
            out.push(ContentBlock::Table {
                table: convert_table_for_structure(t),
                page: 1,
            });
        }
        DocumentChild::StructuredDataTag(sdt) => {
            collect_sdt_blocks(sdt, out);
        }
        _ => {}
    }
}

fn collect_sdt_blocks(sdt: &docx_rs::StructuredDataTag, out: &mut Vec<ContentBlock>) {
    for child in &sdt.children {
        match child {
            StructuredDataTagChild::Paragraph(p) => {
                let text = extract_paragraph_text(p);
                if text.is_empty() {
                    continue;
                }

                if let Some(ref style) = p.property.style {
                    if let Some(level) = heading_level_from_style(&style.val) {
                        out.push(ContentBlock::Heading {
                            text,
                            level,
                            bbox: (0.0, 0.0, 0.0, 0.0),
                            page: 1,
                        });
                        continue;
                    }
                }

                if p.property.numbering_property.is_some() {
                    let indent_level = p
                        .property
                        .numbering_property
                        .as_ref()
                        .and_then(|np| np.level.as_ref())
                        .map(|lvl| lvl.val as u8)
                        .unwrap_or(0);
                    out.push(ContentBlock::ListItem {
                        text,
                        indent_level,
                        bbox: (0.0, 0.0, 0.0, 0.0),
                        page: 1,
                    });
                    continue;
                }

                out.push(ContentBlock::Paragraph {
                    text,
                    bbox: (0.0, 0.0, 0.0, 0.0),
                    page: 1,
                });
            }
            StructuredDataTagChild::Table(t) => {
                out.push(ContentBlock::Table {
                    table: convert_table_for_structure(t),
                    page: 1,
                });
            }
            StructuredDataTagChild::StructuredDataTag(nested) => {
                collect_sdt_blocks(nested, out);
            }
            _ => {}
        }
    }
}

/// Parse a heading level from a paragraph style id.
///
/// Matches common patterns: "Heading1" through "Heading9" and the
/// locale-independent forms "heading 1" etc.
fn heading_level_from_style(style_id: &str) -> Option<u8> {
    let normalized = style_id.to_lowercase().replace(' ', "");
    if let Some(rest) = normalized.strip_prefix("heading") {
        if let Ok(level) = rest.parse::<u8>() {
            if (1..=9).contains(&level) {
                return Some(level);
            }
        }
    }
    None
}

fn convert_table_for_structure(table: &docx_rs::Table) -> Table {
    let mut rows = Vec::new();
    let mut max_cols: usize = 0;

    for row_child in &table.rows {
        let TableChild::TableRow(row) = row_child;
        let mut cells = Vec::new();

        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            let mut parts = Vec::new();
            for content in &cell.children {
                if let docx_rs::TableCellContent::Paragraph(p) = content {
                    let t = extract_paragraph_text(p);
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
            cells.push(Cell {
                text: parts.join("\n"),
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
