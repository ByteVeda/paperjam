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

#[pyfunction]
#[pyo3(name = "rotate_pages")]
pub fn py_rotate_pages(
    py: Python<'_>,
    document: &PyDocument,
    page_rotations: Vec<(u32, i32)>,
) -> PyResult<PyDocument> {
    // Clone the lopdf document directly (no serialize/reparse)
    let inner_clone = document.inner.inner().clone();
    let mut doc =
        paperjam_core::document::Document::from_lopdf(inner_clone).map_err(to_py_err)?;

    let rotations: Vec<(u32, paperjam_core::manipulation::Rotation)> = page_rotations
        .into_iter()
        .map(|(page, degrees)| {
            let rot = match degrees {
                90 => paperjam_core::manipulation::Rotation::Degrees90,
                180 => paperjam_core::manipulation::Rotation::Degrees180,
                270 => paperjam_core::manipulation::Rotation::Degrees270,
                _ => paperjam_core::manipulation::Rotation::Degrees0,
            };
            (page, rot)
        })
        .collect();

    py.allow_threads(move || {
        paperjam_core::manipulation::rotate_pages(&mut doc, &rotations)?;
        Ok::<_, paperjam_core::error::PdfError>(doc)
    })
    .map_err(to_py_err)
    .map(|doc| PyDocument {
        inner: std::sync::Arc::new(doc),
    })
}

#[pyfunction]
#[pyo3(name = "reorder_pages")]
pub fn py_reorder_pages(
    py: Python<'_>,
    document: &PyDocument,
    page_order: Vec<u32>,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);
    let result = py
        .allow_threads(move || paperjam_core::manipulation::reorder_pages(&inner, &page_order))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}
