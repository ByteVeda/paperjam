use scraper::{ElementRef, Selector};

use paperjam_model::structure::ContentBlock;

use crate::document::HtmlDocument;
use crate::error::HtmlError;

/// Extract document structure (headings, paragraphs, lists, tables) from a parsed HTML DOM.
pub fn extract_structure_from_html(dom: &scraper::Html) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    walk_for_structure(dom.root_element(), &mut blocks);
    blocks
}

fn walk_for_structure(element: ElementRef, blocks: &mut Vec<ContentBlock>) {
    for child in element.children() {
        let Some(child_el) = ElementRef::wrap(child) else {
            continue;
        };
        let tag = child_el.value().name().to_ascii_lowercase();

        match tag.as_str() {
            "script" | "style" | "noscript" | "svg" => continue,
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse::<u8>().unwrap_or(1);
                let text = child_el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    blocks.push(ContentBlock::Heading {
                        text,
                        level,
                        bbox: (0.0, 0.0, 0.0, 0.0),
                        page: 1,
                    });
                }
            }
            "p" => {
                let text = child_el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    blocks.push(ContentBlock::Paragraph {
                        text,
                        bbox: (0.0, 0.0, 0.0, 0.0),
                        page: 1,
                    });
                }
            }
            "ul" | "ol" => {
                extract_list_items(child_el, 0, blocks);
            }
            "table" => {
                // Parse just this table element.
                let tables = extract_tables_from_html_element(child_el);
                for table in tables {
                    blocks.push(ContentBlock::Table { table, page: 1 });
                }
            }
            "blockquote" => {
                let text = child_el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    blocks.push(ContentBlock::Paragraph {
                        text,
                        bbox: (0.0, 0.0, 0.0, 0.0),
                        page: 1,
                    });
                }
            }
            // For structural containers, recurse into them.
            "div" | "article" | "section" | "main" | "header" | "footer" | "nav" | "aside"
            | "body" | "html" | "head" => {
                walk_for_structure(child_el, blocks);
            }
            _ => {
                // For unknown elements, try recursing.
                walk_for_structure(child_el, blocks);
            }
        }
    }
}

fn extract_list_items(list_el: ElementRef, depth: usize, blocks: &mut Vec<ContentBlock>) {
    // Iterate direct children manually (scraper doesn't support :scope).
    for child in list_el.children() {
        let Some(child_el) = ElementRef::wrap(child) else {
            continue;
        };
        let tag = child_el.value().name().to_ascii_lowercase();
        if tag == "li" {
            // Collect direct text (not nested list text).
            let text = collect_li_text(child_el);
            if !text.is_empty() {
                blocks.push(ContentBlock::ListItem {
                    text,
                    indent_level: depth as u8,
                    bbox: (0.0, 0.0, 0.0, 0.0),
                    page: 1,
                });
            }

            // Check for nested lists.
            for nested in child_el.children() {
                if let Some(nested_el) = ElementRef::wrap(nested) {
                    let nested_tag = nested_el.value().name().to_ascii_lowercase();
                    if nested_tag == "ul" || nested_tag == "ol" {
                        extract_list_items(nested_el, depth + 1, blocks);
                    }
                }
            }
        }
    }
}

/// Collect text from an `<li>` element, excluding nested list text.
fn collect_li_text(li_el: ElementRef) -> String {
    let mut parts = Vec::new();
    for child in li_el.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
            scraper::node::Node::Element(_) => {
                if let Some(el) = ElementRef::wrap(child) {
                    let tag = el.value().name().to_ascii_lowercase();
                    // Skip nested lists — their items are handled separately.
                    if tag != "ul" && tag != "ol" {
                        let text = el.text().collect::<String>().trim().to_string();
                        if !text.is_empty() {
                            parts.push(text);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    parts.join(" ")
}

/// Parse a single `<table>` element into model tables.
fn extract_tables_from_html_element(table_el: ElementRef) -> Vec<paperjam_model::table::Table> {
    use paperjam_model::table::{Cell, Row, Table, TableStrategy};

    let tr_sel = match Selector::parse("tr") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let td_sel = match Selector::parse("td, th") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

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
        return Vec::new();
    }

    vec![Table {
        bbox: (0.0, 0.0, 0.0, 0.0),
        rows,
        col_count: max_cols,
        strategy: TableStrategy::Auto,
    }]
}

/// Extract structure from HTML with a custom page number (for EPUB chapter reuse).
pub fn extract_structure_from_html_with_page(dom: &scraper::Html, page: u32) -> Vec<ContentBlock> {
    let mut blocks = extract_structure_from_html(dom);
    for block in &mut blocks {
        match block {
            ContentBlock::Heading { page: p, .. }
            | ContentBlock::Paragraph { page: p, .. }
            | ContentBlock::ListItem { page: p, .. }
            | ContentBlock::Table { page: p, .. } => {
                *p = page;
            }
        }
    }
    blocks
}

impl HtmlDocument {
    pub fn extract_structure(&self) -> Result<Vec<ContentBlock>, HtmlError> {
        Ok(extract_structure_from_html(&self.dom))
    }
}
