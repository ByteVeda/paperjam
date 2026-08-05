use paperjam_model::format::DocumentFormat;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::errors::FormatError;

/// Convert a file from one format to another.
#[pyfunction]
#[pyo3(name = "convert_file")]
pub fn py_convert_file(
    py: Python<'_>,
    input_path: String,
    output_path: String,
) -> PyResult<Bound<'_, PyDict>> {
    let report = py.allow_threads(|| {
        paperjam_convert::convert(
            std::path::Path::new(&input_path),
            std::path::Path::new(&output_path),
        )
        .map_err(|e| FormatError::new_err(e.to_string()))
    })?;

    let dict = PyDict::new(py);
    dict.set_item("from_format", report.from_format.display_name())?;
    dict.set_item("to_format", report.to_format.display_name())?;
    dict.set_item("content_blocks", report.content_blocks)?;
    dict.set_item("tables", report.tables)?;
    dict.set_item("images", report.images)?;
    Ok(dict)
}

/// Convert in-memory bytes from one format to another.
#[pyfunction]
#[pyo3(name = "convert_bytes")]
pub fn py_convert_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    from_format: &str,
    to_format: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let from = DocumentFormat::from_extension(from_format);
    let to = DocumentFormat::from_extension(to_format);

    let output = py.allow_threads(|| {
        paperjam_convert::convert_bytes(data, from, to)
            .map_err(|e| FormatError::new_err(e.to_string()))
    })?;

    Ok(PyBytes::new(py, &output))
}

/// Detect the format of a file from its path (extension + magic bytes).
#[pyfunction]
#[pyo3(name = "detect_format")]
pub fn py_detect_format(path: &str) -> String {
    let format = paperjam_convert::detect_format(std::path::Path::new(path));
    format.extension().to_string()
}
