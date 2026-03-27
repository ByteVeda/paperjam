use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "stamp_pages")]
#[pyo3(signature = (document, stamp_document, *, source_page=1, target_pages=None, x=0.0, y=0.0, scale=1.0, opacity=1.0, layer="over"))]
#[allow(clippy::too_many_arguments)]
pub fn py_stamp_pages(
    py: Python<'_>,
    document: &PyDocument,
    stamp_document: &PyDocument,
    source_page: u32,
    target_pages: Option<Vec<u32>>,
    x: f64,
    y: f64,
    scale: f64,
    opacity: f64,
    layer: &str,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);
    let stamp_inner = std::sync::Arc::clone(&stamp_document.inner);
    let layer_enum = paperjam_core::stamp::StampLayer::from_str(layer);

    let options = paperjam_core::stamp::StampOptions {
        source_page,
        target_pages,
        x,
        y,
        scale,
        opacity,
        layer: layer_enum,
    };

    let result = py
        .allow_threads(move || paperjam_core::stamp::stamp_pages(&inner, &stamp_inner, &options))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}
