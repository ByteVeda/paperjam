use crate::error::Result;
use crate::page::Page;
use crate::table::types::*;
use crate::text::layout::TextLine;

/// Extract tables using the stream (whitespace-based) strategy.
pub fn extract_stream_tables(page: &Page, options: &TableExtractionOptions) -> Result<Vec<Table>> {
    let lines = page.text_lines()?;
    if lines.is_empty() {
        return Ok(vec![]);
    }

    let analyzed: Vec<AnalyzedLine> = lines
        .iter()
        .map(|line| analyze_line(line, options.min_col_gap))
        .collect();

    let mode_cols = find_mode_column_count(&analyzed);
    if mode_cols < options.min_cols {
        return Ok(vec![]);
    }

    let col_boundaries = detect_column_boundaries(&analyzed, mode_cols);

    let regions = find_table_regions(&analyzed, &col_boundaries, options);

    let mut tables = Vec::new();
    for region in regions {
        let table = build_table_from_region(&region, &col_boundaries);
        if table.row_count() >= options.min_rows && table.col_count >= options.min_cols {
            tables.push(table);
        }
    }

    Ok(tables)
}

struct AnalyzedLine<'a> {
    line: &'a TextLine,
    words: Vec<(f64, f64, String)>, // (x_start, x_end, text)
}

fn analyze_line<'a>(line: &'a TextLine, min_gap: f64) -> AnalyzedLine<'a> {
    let mut words = Vec::new();
    let mut current_text = String::new();
    let mut current_start = 0.0;
    let mut current_end = 0.0;

    for (i, span) in line.spans.iter().enumerate() {
        if i == 0 {
            current_start = span.x;
            current_end = span.x + span.width;
            current_text = span.text.clone();
        } else {
            let gap = span.x - current_end;
            if gap > min_gap {
                words.push((current_start, current_end, current_text.clone()));
                current_start = span.x;
                current_end = span.x + span.width;
                current_text = span.text.clone();
            } else {
                if gap > span.font_size * 0.25 {
                    current_text.push(' ');
                }
                current_text.push_str(&span.text);
                current_end = span.x + span.width;
            }
        }
    }

    if !current_text.is_empty() {
        words.push((current_start, current_end, current_text));
    }

    AnalyzedLine { line, words }
}

fn find_mode_column_count(lines: &[AnalyzedLine]) -> usize {
    let mut counts = std::collections::HashMap::new();
    for line in lines {
        if !line.words.is_empty() {
            *counts.entry(line.words.len()).or_insert(0usize) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(cols, _)| cols)
        .unwrap_or(0)
}

fn detect_column_boundaries(lines: &[AnalyzedLine], expected_cols: usize) -> Vec<f64> {
    // Collect all word left-edges
    let mut left_edges: Vec<f64> = Vec::new();
    for line in lines {
        if line.words.len() == expected_cols {
            for (x_start, _, _) in &line.words {
                left_edges.push(*x_start);
            }
        }
    }

    if left_edges.is_empty() {
        return Vec::new();
    }

    left_edges.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Simple gap-based clustering
    let mut boundaries = Vec::new();
    let chunk_size = left_edges.len() / expected_cols;
    if chunk_size == 0 {
        return left_edges;
    }

    for i in 0..expected_cols {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(left_edges.len());
        let avg: f64 = left_edges[start..end].iter().sum::<f64>() / (end - start) as f64;
        boundaries.push(avg);
    }

    boundaries
}

fn find_table_regions<'a>(
    lines: &'a [AnalyzedLine<'a>],
    col_boundaries: &[f64],
    options: &TableExtractionOptions,
) -> Vec<Vec<&'a AnalyzedLine<'a>>> {
    let mut regions = Vec::new();
    let mut current_region: Vec<&AnalyzedLine> = Vec::new();

    for line in lines {
        let is_table_line =
            line.words.len() >= options.min_cols && is_aligned(line, col_boundaries);

        if is_table_line {
            current_region.push(line);
        } else if current_region.len() >= options.min_rows {
            regions.push(current_region);
            current_region = Vec::new();
        } else {
            current_region.clear();
        }
    }

    if current_region.len() >= options.min_rows {
        regions.push(current_region);
    }

    regions
}

fn is_aligned(line: &AnalyzedLine, boundaries: &[f64]) -> bool {
    if boundaries.is_empty() || line.words.is_empty() {
        return false;
    }

    let tolerance = 20.0; // points
    let mut aligned_count = 0;

    for (x_start, _, _) in &line.words {
        if boundaries
            .iter()
            .any(|b| (x_start - b).abs() < tolerance)
        {
            aligned_count += 1;
        }
    }

    aligned_count as f64 / line.words.len() as f64 > 0.5
}

fn build_table_from_region(region: &[&AnalyzedLine], col_boundaries: &[f64]) -> Table {
    let num_cols = col_boundaries.len();
    let mut rows = Vec::new();

    for line in region {
        let mut cells: Vec<Cell> = (0..num_cols)
            .map(|_| Cell {
                text: String::new(),
                bbox: (0.0, 0.0, 0.0, 0.0),
                col_span: 1,
                row_span: 1,
            })
            .collect();

        for (x_start, x_end, text) in &line.words {
            let center = (x_start + x_end) / 2.0;
            let col_idx = find_column(center, col_boundaries, num_cols);
            if col_idx < cells.len() {
                if !cells[col_idx].text.is_empty() {
                    cells[col_idx].text.push(' ');
                }
                cells[col_idx].text.push_str(text);
                cells[col_idx].bbox = (*x_start, line.line.bbox.1, *x_end, line.line.bbox.3);
            }
        }

        rows.push(Row {
            cells,
            y_min: line.line.bbox.1,
            y_max: line.line.bbox.3,
        });
    }

    let bbox = if !rows.is_empty() {
        let x_min = col_boundaries.first().copied().unwrap_or(0.0);
        let x_max = rows
            .iter()
            .flat_map(|r| r.cells.iter().map(|c| c.bbox.2))
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = rows.last().map(|r| r.y_min).unwrap_or(0.0);
        let y_max = rows.first().map(|r| r.y_max).unwrap_or(0.0);
        (x_min, y_min, x_max, y_max)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    Table {
        bbox,
        rows,
        col_count: num_cols,
        strategy: TableStrategy::Stream,
    }
}

fn find_column(x: f64, boundaries: &[f64], _num_cols: usize) -> usize {
    for i in (0..boundaries.len()).rev() {
        if x >= boundaries[i] - 10.0 {
            return i;
        }
    }
    0
}
