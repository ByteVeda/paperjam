use crate::text::layout::TextLine;

use super::columns::{ColumnBand, Gutter};

/// Separate lines into header, body, and footer zones.
///
/// Returns (header_lines, body_lines, footer_lines).
pub(crate) fn separate_header_footer(
    lines: &[TextLine],
    page_height: f64,
    header_zone_fraction: f64,
    footer_zone_fraction: f64,
) -> (Vec<&TextLine>, Vec<&TextLine>, Vec<&TextLine>) {
    // In PDF coordinates, Y=0 is bottom, Y=page_height is top.
    let header_y_threshold = page_height * (1.0 - header_zone_fraction);
    let footer_y_threshold = page_height * footer_zone_fraction;

    let mut header = Vec::new();
    let mut body = Vec::new();
    let mut footer = Vec::new();

    for line in lines {
        let y_center = (line.bbox.1 + line.bbox.3) / 2.0;
        if y_center > header_y_threshold {
            header.push(line);
        } else if y_center < footer_y_threshold {
            footer.push(line);
        } else {
            body.push(line);
        }
    }

    // Validate: if header/footer zone has too many lines relative to body,
    // it's probably body text, not a true header/footer.
    let body_count = body.len().max(1);
    let max_header = (body_count as f64 * 0.05).ceil() as usize + 3;
    let max_footer = (body_count as f64 * 0.05).ceil() as usize + 3;

    if header.len() > max_header {
        body.append(&mut header);
        body.sort_by(|a, b| {
            b.bbox
                .3
                .partial_cmp(&a.bbox.3)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    if footer.len() > max_footer {
        body.append(&mut footer);
        body.sort_by(|a, b| {
            b.bbox
                .3
                .partial_cmp(&a.bbox.3)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    (header, body, footer)
}

/// Check if a text line is full-width (spans across gutter boundaries).
pub(crate) fn is_full_width(line: &TextLine, gutters: &[Gutter], page_width: f64) -> bool {
    let line_width = line.bbox.2 - line.bbox.0;

    // Spans more than 70% of page width
    if line_width > page_width * 0.7 {
        return true;
    }

    // Spans across at least one gutter center
    for gutter in gutters {
        if line.bbox.0 < gutter.center && line.bbox.2 > gutter.center {
            // Check that it meaningfully crosses (not just barely touching)
            let overlap_left = gutter.x_start - line.bbox.0;
            let overlap_right = line.bbox.2 - gutter.x_end;
            if overlap_left > 20.0 && overlap_right > 20.0 {
                return true;
            }
        }
    }

    false
}

/// Separate body lines into full-width and per-column lines.
///
/// Returns (full_width_lines, column_lines) where column_lines[i] holds
/// lines assigned to column i.
pub(crate) fn separate_full_width<'a>(
    body_lines: &[&'a TextLine],
    gutters: &[Gutter],
    page_width: f64,
    columns: &[ColumnBand],
) -> (Vec<&'a TextLine>, Vec<Vec<&'a TextLine>>) {
    let mut full_width = Vec::new();
    let mut col_lines: Vec<Vec<&TextLine>> = vec![Vec::new(); columns.len()];

    for &line in body_lines {
        if columns.len() > 1 && is_full_width(line, gutters, page_width) {
            full_width.push(line);
        } else {
            let col_idx = super::columns::assign_to_column(line, columns);
            if col_idx < col_lines.len() {
                col_lines[col_idx].push(line);
            }
        }
    }

    (full_width, col_lines)
}
