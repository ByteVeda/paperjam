use paperjam_core::page::Page;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;

use crate::errors::to_py_err;

#[pyclass(name = "RustPage")]
pub struct PyPage {
    pub(crate) inner: Arc<Page>,
}

#[pymethods]
impl PyPage {
    fn number(&self) -> u32 {
        self.inner.number
    }

    fn width(&self) -> f64 {
        self.inner.width
    }

    fn height(&self) -> f64 {
        self.inner.height
    }

    fn rotation(&self) -> u32 {
        self.inner.rotation
    }

    fn extract_text(&self, py: Python<'_>) -> PyResult<String> {
        let page = Arc::clone(&self.inner);
        py.allow_threads(move || page.extract_text())
            .map_err(to_py_err)
    }

    fn extract_text_spans<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let page = Arc::clone(&self.inner);
        let spans = py
            .allow_threads(move || page.text_spans())
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for span in &spans {
            let dict = PyDict::new(py);
            dict.set_item("text", &span.text)?;
            dict.set_item("x", span.x)?;
            dict.set_item("y", span.y)?;
            dict.set_item("width", span.width)?;
            dict.set_item("font_size", span.font_size)?;
            dict.set_item("font_name", &span.font_name)?;
            list.append(dict)?;
        }
        Ok(list)
    }

    fn extract_text_lines<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let page = Arc::clone(&self.inner);
        let lines = py
            .allow_threads(move || page.text_lines())
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for line in &lines {
            let line_dict = PyDict::new(py);
            line_dict.set_item("text", line.text())?;
            line_dict.set_item("bbox", (line.bbox.0, line.bbox.1, line.bbox.2, line.bbox.3))?;

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

            list.append(line_dict)?;
        }
        Ok(list)
    }

    #[pyo3(signature = (*, strategy="auto", min_rows=2, min_cols=2, snap_tolerance=3.0, row_tolerance=0.5, min_col_gap=10.0))]
    #[allow(clippy::too_many_arguments)]
    fn extract_tables<'py>(
        &self,
        py: Python<'py>,
        strategy: &str,
        min_rows: usize,
        min_cols: usize,
        snap_tolerance: f64,
        row_tolerance: f64,
        min_col_gap: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let page = Arc::clone(&self.inner);
        let opts = paperjam_core::table::TableExtractionOptions {
            strategy: match strategy {
                "lattice" => paperjam_core::table::TableStrategy::Lattice,
                "stream" => paperjam_core::table::TableStrategy::Stream,
                _ => paperjam_core::table::TableStrategy::Auto,
            },
            min_rows,
            min_cols,
            snap_tolerance,
            row_tolerance,
            min_col_gap,
        };

        let tables = py
            .allow_threads(move || page.extract_tables(&opts))
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for table in &tables {
            let table_dict = crate::convert::table_to_py(py, table)?;
            list.append(table_dict)?;
        }
        Ok(list)
    }

    #[pyo3(signature = (*, heading_size_ratio=1.2, detect_lists=true, include_tables=true, layout_aware=false))]
    fn extract_structure<'py>(
        &self,
        py: Python<'py>,
        heading_size_ratio: f64,
        detect_lists: bool,
        include_tables: bool,
        layout_aware: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        let page = Arc::clone(&self.inner);
        let options = paperjam_core::structure::StructureOptions {
            heading_size_ratio,
            detect_lists,
            include_tables,
            layout_aware,
        };
        let blocks = py
            .allow_threads(move || paperjam_core::structure::extract_structure(&page, &options))
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for block in &blocks {
            let dict = crate::convert::content_block_to_py(py, block)?;
            list.append(dict)?;
        }
        Ok(list)
    }

    #[pyo3(signature = (*, min_gutter_width=20.0, max_columns=4, detect_headers_footers=true, header_zone_fraction=0.08, footer_zone_fraction=0.08, min_column_line_fraction=0.1))]
    #[allow(clippy::too_many_arguments)]
    fn analyze_layout<'py>(
        &self,
        py: Python<'py>,
        min_gutter_width: f64,
        max_columns: usize,
        detect_headers_footers: bool,
        header_zone_fraction: f64,
        footer_zone_fraction: f64,
        min_column_line_fraction: f64,
    ) -> PyResult<Bound<'py, PyDict>> {
        let page = Arc::clone(&self.inner);
        let options = paperjam_core::layout::LayoutOptions {
            min_gutter_width,
            max_columns,
            detect_headers_footers,
            header_zone_fraction,
            footer_zone_fraction,
            min_column_line_fraction,
        };
        let layout = py
            .allow_threads(move || paperjam_core::layout::analyze_layout(&page, &options))
            .map_err(to_py_err)?;

        crate::convert::page_layout_to_py(py, &layout)
    }

    #[pyo3(signature = (*, min_gutter_width=20.0, max_columns=4, detect_headers_footers=true, header_zone_fraction=0.08, footer_zone_fraction=0.08, min_column_line_fraction=0.1))]
    #[allow(clippy::too_many_arguments)]
    fn extract_text_layout(
        &self,
        py: Python<'_>,
        min_gutter_width: f64,
        max_columns: usize,
        detect_headers_footers: bool,
        header_zone_fraction: f64,
        footer_zone_fraction: f64,
        min_column_line_fraction: f64,
    ) -> PyResult<String> {
        let page = Arc::clone(&self.inner);
        let options = paperjam_core::layout::LayoutOptions {
            min_gutter_width,
            max_columns,
            detect_headers_footers,
            header_zone_fraction,
            footer_zone_fraction,
            min_column_line_fraction,
        };
        py.allow_threads(move || {
            let layout = paperjam_core::layout::analyze_layout(&page, &options)?;
            Ok(layout.text())
        })
        .map_err(to_py_err)
    }

    #[pyo3(signature = (
        *,
        heading_offset=0,
        page_separator="---",
        include_page_numbers=false,
        page_number_format="<!-- page {n} -->",
        html_tables=false,
        table_header_first_row=true,
        normalize_list_markers=true,
        heading_size_ratio=1.2,
        detect_lists=true,
        include_tables=true,
        layout_aware=false,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn to_markdown(
        &self,
        py: Python<'_>,
        heading_offset: u8,
        page_separator: &str,
        include_page_numbers: bool,
        page_number_format: &str,
        html_tables: bool,
        table_header_first_row: bool,
        normalize_list_markers: bool,
        heading_size_ratio: f64,
        detect_lists: bool,
        include_tables: bool,
        layout_aware: bool,
    ) -> PyResult<String> {
        let page = Arc::clone(&self.inner);
        let options = paperjam_core::markdown::MarkdownOptions {
            heading_offset,
            page_separator: page_separator.to_string(),
            include_page_numbers,
            page_number_format: page_number_format.to_string(),
            html_tables,
            table_header_first_row,
            normalize_list_markers,
            structure_options: paperjam_core::structure::StructureOptions {
                heading_size_ratio,
                detect_lists,
                include_tables,
                layout_aware,
            },
        };
        py.allow_threads(move || paperjam_core::markdown::page_to_markdown(&page, &options))
            .map_err(to_py_err)
    }
}
