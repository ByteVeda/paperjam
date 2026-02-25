use crate::text::layout::TextLine;

/// Horizontal extent of a text line on the page.
#[derive(Debug, Clone)]
pub(crate) struct XExtent {
    pub x_min: f64,
    pub x_max: f64,
}

/// A detected gutter (empty vertical band between columns).
#[derive(Debug, Clone)]
pub(crate) struct Gutter {
    pub x_start: f64,
    pub x_end: f64,
    pub center: f64,
}

/// A detected column band.
#[derive(Debug, Clone)]
pub(crate) struct ColumnBand {
    pub x_min: f64,
    pub x_max: f64,
}

/// Extract x-extents from text lines.
pub(crate) fn extract_extents(lines: &[TextLine]) -> Vec<XExtent> {
    lines
        .iter()
        .filter(|l| !l.spans.is_empty())
        .map(|l| XExtent {
            x_min: l.bbox.0,
            x_max: l.bbox.2,
        })
        .collect()
}

/// Build a 1D histogram of text coverage along the X axis.
pub(crate) fn build_x_projection(
    extents: &[XExtent],
    page_width: f64,
    bin_size: f64,
) -> Vec<usize> {
    let num_bins = (page_width / bin_size).ceil() as usize;
    let mut profile = vec![0usize; num_bins.max(1)];

    for ext in extents {
        let start_bin = (ext.x_min / bin_size).floor().max(0.0) as usize;
        let end_bin = ((ext.x_max / bin_size).ceil() as usize).min(num_bins);
        for bin in start_bin..end_bin {
            profile[bin] += 1;
        }
    }
    profile
}

/// Find gutters (empty vertical bands) in the projection profile.
pub(crate) fn find_gutters(
    profile: &[usize],
    bin_size: f64,
    min_gutter_width: f64,
    page_width: f64,
    total_lines: usize,
) -> Vec<Gutter> {
    if profile.is_empty() || total_lines == 0 {
        return Vec::new();
    }

    // A bin counts as "empty" if its count is below a noise threshold
    let empty_threshold = (total_lines as f64 * 0.02).ceil() as usize;

    // Skip page margins (first/last 5% of width)
    let margin_bins = (page_width * 0.05 / bin_size) as usize;
    let start = margin_bins;
    let end = profile.len().saturating_sub(margin_bins);

    if start >= end {
        return Vec::new();
    }

    let mut gutters = Vec::new();
    let mut run_start: Option<usize> = None;

    for i in start..end {
        if profile[i] <= empty_threshold {
            if run_start.is_none() {
                run_start = Some(i);
            }
        } else {
            if let Some(rs) = run_start {
                let width = (i - rs) as f64 * bin_size;
                if width >= min_gutter_width {
                    let x_start = rs as f64 * bin_size;
                    let x_end = i as f64 * bin_size;
                    gutters.push(Gutter {
                        x_start,
                        x_end,
                        center: (x_start + x_end) / 2.0,
                    });
                }
            }
            run_start = None;
        }
    }

    // Check trailing run
    if let Some(rs) = run_start {
        let width = (end - rs) as f64 * bin_size;
        if width >= min_gutter_width {
            let x_start = rs as f64 * bin_size;
            let x_end = end as f64 * bin_size;
            gutters.push(Gutter {
                x_start,
                x_end,
                center: (x_start + x_end) / 2.0,
            });
        }
    }

    gutters
}

/// Determine column bands from detected gutters.
pub(crate) fn determine_columns(
    gutters: &[Gutter],
    extents: &[XExtent],
    page_width: f64,
    min_column_line_fraction: f64,
    max_columns: usize,
) -> Vec<ColumnBand> {
    if gutters.is_empty() {
        return vec![ColumnBand {
            x_min: 0.0,
            x_max: page_width,
        }];
    }

    // Build bands from gutters
    let mut bands = Vec::new();
    let mut left = 0.0_f64;
    for gutter in gutters {
        bands.push(ColumnBand {
            x_min: left,
            x_max: gutter.center,
        });
        left = gutter.center;
    }
    bands.push(ColumnBand {
        x_min: left,
        x_max: page_width,
    });

    // Validate: each column must contain a minimum fraction of lines
    let min_lines = (extents.len() as f64 * min_column_line_fraction).max(1.0) as usize;

    bands.retain(|band| {
        let count = extents
            .iter()
            .filter(|e| {
                let center = (e.x_min + e.x_max) / 2.0;
                center >= band.x_min && center < band.x_max
            })
            .count();
        count >= min_lines
    });

    // Limit to max_columns
    bands.truncate(max_columns);

    if bands.is_empty() {
        vec![ColumnBand {
            x_min: 0.0,
            x_max: page_width,
        }]
    } else {
        bands
    }
}

/// Assign a text line to the nearest column by x-center.
pub(crate) fn assign_to_column(line: &TextLine, columns: &[ColumnBand]) -> usize {
    if columns.len() <= 1 {
        return 0;
    }

    let x_center = (line.bbox.0 + line.bbox.2) / 2.0;

    columns
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let mid_a = (a.x_min + a.x_max) / 2.0;
            let mid_b = (b.x_min + b.x_max) / 2.0;
            let dist_a = (x_center - mid_a).abs();
            let dist_b = (x_center - mid_b).abs();
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}
