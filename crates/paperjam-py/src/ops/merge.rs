use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "merge")]
pub fn py_merge(
    py: Python<'_>,
    documents: Vec<PyRef<'_, PyDocument>>,
    deduplicate_resources: bool,
) -> PyResult<PyDocument> {
    let docs: paperjam_core::error::Result<Vec<paperjam_core::document::Document>> = documents
        .iter()
        .map(|d| {
            // Clone lopdf document directly (no serialize/reparse)
            let inner_clone = d.inner.inner().clone();
            paperjam_core::document::Document::from_lopdf(inner_clone)
        })
        .collect();

    let docs = docs.map_err(to_py_err)?;
    let options = paperjam_core::manipulation::MergeOptions {
        deduplicate_resources,
    };

    let merged = py
        .allow_threads(move || paperjam_core::manipulation::merge(docs, &options))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(merged),
    })
}

#[pyfunction]
#[pyo3(name = "split")]
pub fn py_split(
    py: Python<'_>,
    document: &PyDocument,
    ranges: Vec<(u32, u32)>,
) -> PyResult<Vec<PyDocument>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let results = py
        .allow_threads(move || paperjam_core::manipulation::split(&inner, &ranges))
        .map_err(to_py_err)?;

    Ok(results
        .into_iter()
        .map(|doc| PyDocument {
            inner: std::sync::Arc::new(doc),
        })
        .collect())
}
