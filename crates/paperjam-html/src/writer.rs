use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;

/// Generate a complete HTML document from content blocks and metadata.
///
/// This produces clean HTML5 suitable for use as a conversion target.
pub fn generate_html_bytes(
    blocks: &[ContentBlock],
    _tables: &[Table],
    metadata: &Metadata,
) -> Vec<u8> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");

    if let Some(title) = &metadata.title {
        html.push_str(&format!("  <title>{}</title>\n", escape_html(title)));
    } else {
        html.push_str("  <title>Document</title>\n");
    }
    if let Some(author) = &metadata.author {
        html.push_str(&format!(
            "  <meta name=\"author\" content=\"{}\">\n",
            escape_attr(author)
        ));
    }
    if let Some(subject) = &metadata.subject {
        html.push_str(&format!(
            "  <meta name=\"description\" content=\"{}\">\n",
            escape_attr(subject)
        ));
    }
    if let Some(keywords) = &metadata.keywords {
        html.push_str(&format!(
            "  <meta name=\"keywords\" content=\"{}\">\n",
            escape_attr(keywords)
        ));
    }

    html.push_str("</head>\n<body>\n");

    let mut in_list = false;
    let mut list_depth: u8 = 0;

    for block in blocks {
        match block {
            ContentBlock::Heading { text, level, .. } => {
                if in_list {
                    close_lists(&mut html, list_depth);
                    in_list = false;
                    list_depth = 0;
                }
                let l = (*level).clamp(1, 6);
                html.push_str(&format!("  <h{l}>{}</h{l}>\n", escape_html(text)));
            }
            ContentBlock::Paragraph { text, .. } => {
                if in_list {
                    close_lists(&mut html, list_depth);
                    in_list = false;
                    list_depth = 0;
                }
                html.push_str(&format!("  <p>{}</p>\n", escape_html(text)));
            }
            ContentBlock::ListItem {
                text, indent_level, ..
            } => {
                let target_depth = *indent_level + 1;
                if !in_list {
                    html.push_str("  <ul>\n");
                    in_list = true;
                    list_depth = 1;
                }
                while list_depth < target_depth {
                    html.push_str(&format!("{}<ul>\n", "  ".repeat(list_depth as usize + 1)));
                    list_depth += 1;
                }
                while list_depth > target_depth {
                    list_depth -= 1;
                    html.push_str(&format!("{}</ul>\n", "  ".repeat(list_depth as usize + 1)));
                }
                html.push_str(&format!(
                    "{}<li>{}</li>\n",
                    "  ".repeat(list_depth as usize + 1),
                    escape_html(text)
                ));
            }
            ContentBlock::Table { table, .. } => {
                if in_list {
                    close_lists(&mut html, list_depth);
                    in_list = false;
                    list_depth = 0;
                }
                write_table(&mut html, table);
            }
        }
    }

    if in_list {
        close_lists(&mut html, list_depth);
    }

    html.push_str("</body>\n</html>\n");
    html.into_bytes()
}

fn close_lists(html: &mut String, depth: u8) {
    for d in (0..depth).rev() {
        html.push_str(&format!("{}</ul>\n", "  ".repeat(d as usize + 1)));
    }
}

fn write_table(html: &mut String, table: &Table) {
    let grid = table.to_vec();
    if grid.is_empty() {
        return;
    }

    html.push_str("  <table>\n");

    // First row as thead.
    if let Some(header) = grid.first() {
        html.push_str("    <thead>\n      <tr>\n");
        for cell in header {
            html.push_str(&format!("        <th>{}</th>\n", escape_html(cell)));
        }
        html.push_str("      </tr>\n    </thead>\n");
    }

    // Remaining rows as tbody.
    if grid.len() > 1 {
        html.push_str("    <tbody>\n");
        for row in grid.iter().skip(1) {
            html.push_str("      <tr>\n");
            for cell in row {
                html.push_str(&format!("        <td>{}</td>\n", escape_html(cell)));
            }
            html.push_str("      </tr>\n");
        }
        html.push_str("    </tbody>\n");
    }

    html.push_str("  </table>\n");
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_html(s).replace('"', "&quot;")
}
