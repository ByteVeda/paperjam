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
}
