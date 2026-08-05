use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "optimize")]
pub fn py_optimize<'py>(
    py: Python<'py>,
    document: &PyDocument,
    compress_streams: bool,
    remove_unused: bool,
    remove_duplicates: bool,
    strip_metadata: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);
    let options = paperjam_core::optimization::OptimizeOptions {
        compress_streams,
        remove_unused_objects: remove_unused,
        remove_duplicates,
        strip_metadata,
    };

    let (optimized, result) = py
        .allow_threads(move || paperjam_core::optimization::optimize(&inner, &options))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("original_size", result.original_size)?;
    dict.set_item("optimized_size", result.optimized_size)?;
    dict.set_item("objects_removed", result.objects_removed)?;
    dict.set_item("streams_compressed", result.streams_compressed)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(optimized),
        },
        dict,
    ))
}
