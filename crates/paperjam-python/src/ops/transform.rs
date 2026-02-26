//! PyO3 bindings for page transformation operations (rotate, reorder, delete, insert).

use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

/// Rotate pages by specified angles.
///
/// `page_rotations` is a list of `(page_number, degrees)` tuples where
/// `page_number` is 1-indexed and `degrees` is 0, 90, 180, or 270.
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

/// Reorder pages by a list of 1-indexed page numbers.
///
/// The output document contains only the pages listed, in the given order.
/// Pages can be repeated (duplicated) or omitted (dropped).
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

/// Delete specific pages from the document.
///
/// `page_numbers` is a list of 1-indexed page numbers to remove.
/// At least one page must remain after deletion.
#[pyfunction]
#[pyo3(name = "delete_pages")]
pub fn py_delete_pages(
    py: Python<'_>,
    document: &PyDocument,
    page_numbers: Vec<u32>,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);
    let result = py
        .allow_threads(move || paperjam_core::manipulation::delete_pages(&inner, &page_numbers))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}

/// Insert blank pages at specified positions.
///
/// `insertions` is a list of `(after_page, width, height)` tuples.
/// `after_page=0` inserts at the beginning; width/height are in PDF points.
#[pyfunction]
#[pyo3(name = "insert_blank_pages")]
pub fn py_insert_blank_pages(
    py: Python<'_>,
    document: &PyDocument,
    insertions: Vec<(u32, f64, f64)>,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);
    let result = py
        .allow_threads(move || {
            paperjam_core::manipulation::insert_blank_pages(&inner, &insertions)
        })
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}
