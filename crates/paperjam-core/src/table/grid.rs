use crate::error::Result;
use crate::table::types::*;
use crate::text::layout::TextSpan;

/// Build a table from a cluster of intersection points and text spans.
pub fn build_from_intersections(
    intersections: &[(f64, f64)],
    text_spans: &[TextSpan],
    options: &TableExtractionOptions,
) -> Result<Option<Table>> {
    if intersections.len() < 4 {
        return Ok(None);
    }

    // Extract unique sorted X and Y coordinates
    let mut xs: Vec<f64> = intersections.iter().map(|p| p.0).collect();
    let mut ys: Vec<f64> = intersections.iter().map(|p| p.1).collect();

    // `total_cmp` gives a total ordering even in the presence of NaN coords
    // extracted from malformed PDFs, where `partial_cmp` would panic.
    xs.sort_by(|a, b| a.total_cmp(b));
    xs.dedup_by(|a, b| (*a - *b).abs() < options.snap_tolerance);

    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() < options.snap_tolerance);

    if xs.len() < 2 || ys.len() < 2 {
        return Ok(None);
    }

    let num_cols = xs.len() - 1;
    let num_rows = ys.len() - 1;

    let mut rows = Vec::new();

    // Build rows from bottom to top (PDF Y-axis goes up)
    for row_idx in 0..num_rows {
        let y_min = ys[row_idx];
        let y_max = ys[row_idx + 1];

        let mut cells = Vec::new();
        for col_idx in 0..num_cols {
            let x_min = xs[col_idx];
            let x_max = xs[col_idx + 1];

            // Find text spans whose center falls within this cell
            let cell_text: String = text_spans
                .iter()
                .filter(|span| {
                    let cx = span.x + span.width / 2.0;
                    let cy = span.y;
                    cx >= x_min && cx <= x_max && cy >= y_min && cy <= y_max
                })
                .map(|span| span.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            cells.push(Cell {
                text: cell_text.trim().to_string(),
                bbox: (x_min, y_min, x_max, y_max),
                col_span: 1,
                row_span: 1,
            });
        }

        rows.push(Row {
            cells,
            y_min,
            y_max,
        });
    }

    // Reverse so first row is top of page
    rows.reverse();

    // Both `xs` and `ys` have len >= 2 here (guarded above), so direct
    // indexing is safe and avoids the `unwrap` pattern.
    let bbox = (xs[0], ys[0], xs[xs.len() - 1], ys[ys.len() - 1]);

    Ok(Some(Table {
        bbox,
        rows,
        col_count: num_cols,
        strategy: TableStrategy::Lattice,
    }))
}
