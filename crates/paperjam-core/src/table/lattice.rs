use crate::error::Result;
use crate::page::Page;
use crate::table::types::*;

/// Extract tables using the lattice (line-based) strategy.
///
/// Scans content stream for line-drawing operators, finds intersections,
/// and builds a grid of cells.
pub fn extract_lattice_tables(page: &Page, options: &TableExtractionOptions) -> Result<Vec<Table>> {
    let lines = extract_line_segments(page)?;

    let h_lines: Vec<LineSegment> = lines
        .iter()
        .filter(|l| l.is_horizontal(options.snap_tolerance))
        .cloned()
        .collect();
    let v_lines: Vec<LineSegment> = lines
        .iter()
        .filter(|l| l.is_vertical(options.snap_tolerance))
        .cloned()
        .collect();

    if h_lines.len() < 2 || v_lines.len() < 2 {
        return Ok(vec![]);
    }

    let intersections = find_intersections(&h_lines, &v_lines, options.snap_tolerance);
    let table_regions = cluster_into_tables(&intersections, options.snap_tolerance);

    let text_spans = page.text_spans()?;
    let mut tables = Vec::new();

    for region in table_regions {
        if let Some(table) =
            super::grid::build_from_intersections(&region, &text_spans, options)?
        {
            if table.row_count() >= options.min_rows && table.col_count >= options.min_cols {
                tables.push(table);
            }
        }
    }

    Ok(tables)
}

#[derive(Debug, Clone)]
struct LineSegment {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl LineSegment {
    fn is_horizontal(&self, tolerance: f64) -> bool {
        (self.y1 - self.y2).abs() < tolerance
    }

    fn is_vertical(&self, tolerance: f64) -> bool {
        (self.x1 - self.x2).abs() < tolerance
    }
}

fn extract_line_segments(page: &Page) -> Result<Vec<LineSegment>> {
    use crate::text::operators::{parse_content_stream, ContentOperator};

    let ops = parse_content_stream(page.content_bytes())?;
    let mut segments = Vec::new();
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut path_start_x = 0.0;
    let mut path_start_y = 0.0;
    let mut path_segments: Vec<LineSegment> = Vec::new();

    for op in &ops {
        match op {
            ContentOperator::MoveTo { x, y } => {
                current_x = *x;
                current_y = *y;
                path_start_x = *x;
                path_start_y = *y;
            }
            ContentOperator::LineTo { x, y } => {
                path_segments.push(LineSegment {
                    x1: current_x,
                    y1: current_y,
                    x2: *x,
                    y2: *y,
                });
                current_x = *x;
                current_y = *y;
            }
            ContentOperator::Rectangle { x, y, w, h } => {
                // Rectangle = 4 line segments
                path_segments.push(LineSegment { x1: *x, y1: *y, x2: x + w, y2: *y });
                path_segments.push(LineSegment { x1: x + w, y1: *y, x2: x + w, y2: y + h });
                path_segments.push(LineSegment { x1: x + w, y1: y + h, x2: *x, y2: y + h });
                path_segments.push(LineSegment { x1: *x, y1: y + h, x2: *x, y2: *y });
            }
            ContentOperator::ClosePath => {
                if (current_x - path_start_x).abs() > 0.01
                    || (current_y - path_start_y).abs() > 0.01
                {
                    path_segments.push(LineSegment {
                        x1: current_x,
                        y1: current_y,
                        x2: path_start_x,
                        y2: path_start_y,
                    });
                }
                current_x = path_start_x;
                current_y = path_start_y;
            }
            ContentOperator::Stroke | ContentOperator::CloseAndStroke | ContentOperator::Fill | ContentOperator::FillEvenOdd => {
                segments.append(&mut path_segments);
            }
            _ => {}
        }
    }

    Ok(segments)
}

fn find_intersections(
    h_lines: &[LineSegment],
    v_lines: &[LineSegment],
    snap: f64,
) -> Vec<(f64, f64)> {
    let mut points = Vec::new();

    for h in h_lines {
        let h_y = (h.y1 + h.y2) / 2.0;
        let h_x_min = h.x1.min(h.x2) - snap;
        let h_x_max = h.x1.max(h.x2) + snap;

        for v in v_lines {
            let v_x = (v.x1 + v.x2) / 2.0;
            let v_y_min = v.y1.min(v.y2) - snap;
            let v_y_max = v.y1.max(v.y2) + snap;

            if v_x >= h_x_min && v_x <= h_x_max && h_y >= v_y_min && h_y <= v_y_max {
                points.push((snap_value(v_x, snap), snap_value(h_y, snap)));
            }
        }
    }

    // Deduplicate
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap()));
    points.dedup_by(|a, b| (a.0 - b.0).abs() < snap && (a.1 - b.1).abs() < snap);

    points
}

fn snap_value(v: f64, snap: f64) -> f64 {
    (v / snap).round() * snap
}

fn cluster_into_tables(
    intersections: &[(f64, f64)],
    snap: f64,
) -> Vec<Vec<(f64, f64)>> {
    if intersections.is_empty() {
        return vec![];
    }

    // Simple clustering: group intersections that share X or Y coordinates
    let mut visited = vec![false; intersections.len()];
    let mut clusters = Vec::new();

    for i in 0..intersections.len() {
        if visited[i] {
            continue;
        }

        let mut cluster = Vec::new();
        let mut stack = vec![i];

        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            cluster.push(intersections[idx]);

            // Find connected intersections (share X or Y within snap tolerance)
            for j in 0..intersections.len() {
                if !visited[j] {
                    let dx = (intersections[idx].0 - intersections[j].0).abs();
                    let dy = (intersections[idx].1 - intersections[j].1).abs();
                    if dx < snap || dy < snap {
                        stack.push(j);
                    }
                }
            }
        }

        if cluster.len() >= 4 {
            // Need at least 4 intersections to form a cell
            clusters.push(cluster);
        }
    }

    clusters
}
