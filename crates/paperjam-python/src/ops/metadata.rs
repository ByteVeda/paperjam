use std::collections::HashMap;

use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "set_metadata", signature = (document, updates))]
pub fn py_set_metadata(
    py: Python<'_>,
    document: &PyDocument,
    updates: HashMap<String, Option<String>>,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);

    let mut meta_update = paperjam_core::metadata::MetadataUpdate::default();

    for (key, value) in &updates {
        let field = Some(value.clone());
        match key.as_str() {
            "title" => meta_update.title = field,
            "author" => meta_update.author = field,
            "subject" => meta_update.subject = field,
            "keywords" => meta_update.keywords = field,
            "creator" => meta_update.creator = field,
            "producer" => meta_update.producer = field,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown metadata field: '{}'",
                    key
                )));
            }
        }
    }

    let result = py
        .allow_threads(move || paperjam_core::metadata::set_metadata(&inner, &meta_update))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}
