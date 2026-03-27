use paperjam_core::layout::{PageLayout, RegionKind};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert a Rust PageLayout to a Python dict.
pub fn page_layout_to_py<'py>(
    py: Python<'py>,
    layout: &PageLayout,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("page_width", layout.page_width)?;
    dict.set_item("page_height", layout.page_height)?;
    dict.set_item("column_count", layout.column_count)?;

    let gutters_list = PyList::empty(py);
    for g in &layout.gutters {
        gutters_list.append(*g)?;
    }
    dict.set_item("gutters", gutters_list)?;

    let regions_list = PyList::empty(py);
    for region in &layout.regions {
        let region_dict = PyDict::new(py);
        let (kind_str, col_idx): (&str, Option<usize>) = match &region.kind {
            RegionKind::Header => ("header", None),
            RegionKind::Footer => ("footer", None),
            RegionKind::BodyColumn { index } => ("body_column", Some(*index)),
            RegionKind::FullWidth => ("full_width", None),
        };
        region_dict.set_item("kind", kind_str)?;
        region_dict.set_item("column_index", col_idx.map(|i| i as u32))?;
        region_dict.set_item(
            "bbox",
            (region.bbox.0, region.bbox.1, region.bbox.2, region.bbox.3),
        )?;

        // Serialize lines (same format as extract_text_lines)
        let lines_list = PyList::empty(py);
        for line in &region.lines {
            let line_dict = PyDict::new(py);
            line_dict.set_item("text", line.text())?;
            line_dict.set_item(
                "bbox",
                (line.bbox.0, line.bbox.1, line.bbox.2, line.bbox.3),
            )?;

            let spans_list = PyList::empty(py);
            for span in &line.spans {
                let span_dict = PyDict::new(py);
                span_dict.set_item("text", &span.text)?;
                span_dict.set_item("x", span.x)?;
                span_dict.set_item("y", span.y)?;
                span_dict.set_item("width", span.width)?;
                span_dict.set_item("font_size", span.font_size)?;
                span_dict.set_item("font_name", &span.font_name)?;
                spans_list.append(span_dict)?;
            }
            line_dict.set_item("spans", spans_list)?;
            lines_list.append(line_dict)?;
        }
        region_dict.set_item("lines", lines_list)?;
        regions_list.append(region_dict)?;
    }
    dict.set_item("regions", regions_list)?;

    Ok(dict)
}
