use crate::text::layout::TextLine;

use super::{LayoutRegion, RegionKind};

fn compute_bbox_owned(lines: &[TextLine]) -> (f64, f64, f64, f64) {
    if lines.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let x_min = lines.iter().map(|l| l.bbox.0).fold(f64::INFINITY, f64::min);
    let y_min = lines.iter().map(|l| l.bbox.1).fold(f64::INFINITY, f64::min);
    let x_max = lines
        .iter()
        .map(|l| l.bbox.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_max = lines
        .iter()
        .map(|l| l.bbox.3)
        .fold(f64::NEG_INFINITY, f64::max);
    (x_min, y_min, x_max, y_max)
}

/// A tagged line for sorting: either full-width or belonging to a column.
#[derive(Debug)]
enum TaggedLine {
    FullWidth {
        y_top: f64,
        line: TextLine,
    },
    Column {
        col: usize,
        y_top: f64,
        line: TextLine,
    },
}

/// Assemble layout regions in correct reading order.
///
/// Order: Header → (alternating FullWidth / Column sections) → Footer
pub(crate) fn compute_reading_order(
    header_lines: Vec<&TextLine>,
    full_width_lines: Vec<&TextLine>,
    column_lines: Vec<Vec<&TextLine>>,
    footer_lines: Vec<&TextLine>,
    num_columns: usize,
) -> Vec<LayoutRegion> {
    let mut regions = Vec::new();

    // 1. Header
    if !header_lines.is_empty() {
        let mut lines: Vec<TextLine> = header_lines.into_iter().cloned().collect();
        sort_lines_top_down(&mut lines);
        let bbox = compute_bbox_owned(&lines);
        regions.push(LayoutRegion {
            kind: RegionKind::Header,
            bbox,
            lines,
        });
    }

    // 2. Body: interleave full-width and column sections by vertical position
    if num_columns <= 1 {
        // Single column: just emit everything in order
        let mut all_lines: Vec<TextLine> = Vec::new();
        for line in &full_width_lines {
            all_lines.push((*line).clone());
        }
        for col in &column_lines {
            for line in col {
                all_lines.push((*line).clone());
            }
        }
        sort_lines_top_down(&mut all_lines);
        if !all_lines.is_empty() {
            let bbox = compute_bbox_owned(&all_lines);
            regions.push(LayoutRegion {
                kind: RegionKind::BodyColumn { index: 0 },
                bbox,
                lines: all_lines,
            });
        }
    } else {
        // Multi-column: merge all lines with tags, sort by Y, segment into sections
        let mut tagged: Vec<TaggedLine> = Vec::new();

        for &line in &full_width_lines {
            tagged.push(TaggedLine::FullWidth {
                y_top: line.bbox.3,
                line: line.clone(),
            });
        }

        for (col_idx, col) in column_lines.iter().enumerate() {
            for &line in col {
                tagged.push(TaggedLine::Column {
                    col: col_idx,
                    y_top: line.bbox.3,
                    line: line.clone(),
                });
            }
        }

        // Sort by Y descending (top-first in PDF coordinates)
        tagged.sort_by(|a, b| {
            let y_a = match a {
                TaggedLine::FullWidth { y_top, .. } | TaggedLine::Column { y_top, .. } => *y_top,
            };
            let y_b = match b {
                TaggedLine::FullWidth { y_top, .. } | TaggedLine::Column { y_top, .. } => *y_top,
            };
            y_b.partial_cmp(&y_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Walk through tagged lines, grouping into sections
        let mut column_accum: Vec<Vec<TextLine>> = vec![Vec::new(); num_columns];

        for tl in tagged {
            match tl {
                TaggedLine::FullWidth { line, .. } => {
                    // Flush accumulated column lines before this full-width section
                    flush_columns(&mut regions, &mut column_accum, num_columns);

                    // Emit full-width region
                    let bbox = compute_bbox_owned(std::slice::from_ref(&line));
                    regions.push(LayoutRegion {
                        kind: RegionKind::FullWidth,
                        bbox,
                        lines: vec![line],
                    });
                }
                TaggedLine::Column { col, line, .. } => {
                    if col < column_accum.len() {
                        column_accum[col].push(line);
                    }
                }
            }
        }

        // Flush remaining column lines
        flush_columns(&mut regions, &mut column_accum, num_columns);
    }

    // 3. Footer
    if !footer_lines.is_empty() {
        let mut lines: Vec<TextLine> = footer_lines.into_iter().cloned().collect();
        sort_lines_top_down(&mut lines);
        let bbox = compute_bbox_owned(&lines);
        regions.push(LayoutRegion {
            kind: RegionKind::Footer,
            bbox,
            lines,
        });
    }

    regions
}

/// Flush accumulated column lines into regions (left-to-right, each top-to-bottom).
fn flush_columns(
    regions: &mut Vec<LayoutRegion>,
    column_accum: &mut [Vec<TextLine>],
    num_columns: usize,
) {
    let has_content = column_accum.iter().any(|c| !c.is_empty());
    if !has_content {
        return;
    }

    for col_idx in 0..num_columns {
        if col_idx >= column_accum.len() {
            break;
        }
        let mut lines = std::mem::take(&mut column_accum[col_idx]);
        if lines.is_empty() {
            continue;
        }
        sort_lines_top_down(&mut lines);
        let bbox = compute_bbox_owned(&lines);
        regions.push(LayoutRegion {
            kind: RegionKind::BodyColumn { index: col_idx },
            bbox,
            lines,
        });
    }
}

/// Sort lines top-to-bottom (Y descending in PDF coordinates).
fn sort_lines_top_down(lines: &mut [TextLine]) {
    lines.sort_by(|a, b| {
        b.bbox
            .3
            .partial_cmp(&a.bbox.3)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
