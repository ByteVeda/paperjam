use paperjam_model::structure::ContentBlock;

use crate::document::HtmlDocument;
use crate::error::HtmlError;

impl HtmlDocument {
    /// Convert the document to Markdown.
    pub fn to_markdown(&self) -> Result<String, HtmlError> {
        let blocks = self.extract_structure()?;
        Ok(blocks_to_markdown(&blocks))
    }
}

fn blocks_to_markdown(blocks: &[ContentBlock]) -> String {
    let mut parts = Vec::new();
    for block in blocks {
        match block {
            ContentBlock::Heading { text, level, .. } => {
                let hashes = "#".repeat(*level as usize);
                parts.push(format!("{} {}", hashes, text));
            }
            ContentBlock::Paragraph { text, .. } => {
                parts.push(text.clone());
            }
            ContentBlock::ListItem {
                text, indent_level, ..
            } => {
                let indent = "  ".repeat(*indent_level as usize);
                parts.push(format!("{}- {}", indent, text));
            }
            ContentBlock::Table { table, .. } => {
                parts.push(table_to_markdown(table));
            }
        }
    }
    parts.join("\n\n")
}

fn table_to_markdown(table: &paperjam_model::table::Table) -> String {
    let grid = table.to_vec();
    if grid.is_empty() {
        return String::new();
    }

    let col_count = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return String::new();
    }

    let mut lines = Vec::new();

    // Header row.
    let header = &grid[0];
    let header_cells: Vec<String> = (0..col_count)
        .map(|i| {
            header
                .get(i)
                .cloned()
                .unwrap_or_default()
                .replace('|', "\\|")
        })
        .collect();
    lines.push(format!("| {} |", header_cells.join(" | ")));

    // Separator.
    let sep: Vec<&str> = (0..col_count).map(|_| "---").collect();
    lines.push(format!("| {} |", sep.join(" | ")));

    // Data rows.
    for row in grid.iter().skip(1) {
        let cells: Vec<String> = (0..col_count)
            .map(|i| row.get(i).cloned().unwrap_or_default().replace('|', "\\|"))
            .collect();
        lines.push(format!("| {} |", cells.join(" | ")));
    }

    lines.join("\n")
}
