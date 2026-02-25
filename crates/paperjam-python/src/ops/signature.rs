use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "sign_document", signature = (document, private_key, certificates, reason=None, location=None, contact_info=None, field_name="Signature1"))]
pub fn py_sign_document<'py>(
    py: Python<'py>,
    document: &PyDocument,
    private_key: &[u8],
    certificates: Vec<Vec<u8>>,
    reason: Option<String>,
    location: Option<String>,
    contact_info: Option<String>,
    field_name: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let pk = private_key.to_vec();
    let certs = certificates;
    let options = paperjam_core::signature::SignOptions {
        reason,
        location,
        contact_info,
        field_name: field_name.to_string(),
    };

    let signed_bytes = py
        .allow_threads(move || {
            paperjam_core::signature::sign_document(&inner, &pk, &certs, &options)
        })
        .map_err(to_py_err)?;

    Ok(PyBytes::new(py, &signed_bytes))
}

#[pyfunction]
#[pyo3(name = "extract_signatures")]
pub fn py_extract_signatures<'py>(
    py: Python<'py>,
    document: &PyDocument,
    raw_bytes: &[u8],
) -> PyResult<Bound<'py, PyList>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let bytes = raw_bytes.to_vec();

    let sigs = py
        .allow_threads(move || {
            paperjam_core::signature::extract_signatures(inner.inner(), &bytes)
        })
        .map_err(to_py_err)?;

    let list = PyList::empty(py);
    for sig in &sigs {
        let dict = crate::convert::signature_info_to_py(py, sig)?;
        list.append(dict)?;
    }
    Ok(list)
}

#[pyfunction]
#[pyo3(name = "verify_signatures")]
pub fn py_verify_signatures<'py>(
    py: Python<'py>,
    document: &PyDocument,
    raw_bytes: &[u8],
) -> PyResult<Bound<'py, PyList>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let bytes = raw_bytes.to_vec();

    let results = py
        .allow_threads(move || {
            paperjam_core::signature::verify_signatures(inner.inner(), &bytes)
        })
        .map_err(to_py_err)?;

    let list = PyList::empty(py);
    for result in &results {
        let dict = crate::convert::signature_validity_to_py(py, result)?;
        list.append(dict)?;
    }
    Ok(list)
}
