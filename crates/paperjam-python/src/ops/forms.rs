use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "fill_form", signature = (document, values, need_appearances=true))]
pub fn py_fill_form<'py>(
    py: Python<'py>,
    document: &PyDocument,
    values: &Bound<'py, PyDict>,
    need_appearances: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let mut field_values = HashMap::new();
    for (key, val) in values.iter() {
        let k: String = key.extract()?;
        let v: String = val.extract()?;
        field_values.insert(k, v);
    }

    let options = paperjam_core::forms::types::FillFormOptions {
        need_appearances,
    };

    let (filled_doc, result) = py
        .allow_threads(move || {
            paperjam_core::forms::fill_form_fields(&inner, &field_values, &options)
        })
        .map_err(to_py_err)?;

    let result_dict = PyDict::new(py);
    result_dict.set_item("fields_filled", result.fields_filled)?;
    result_dict.set_item("fields_not_found", result.fields_not_found)?;

    let not_found_list = PyList::empty(py);
    for name in &result.not_found_names {
        not_found_list.append(name)?;
    }
    result_dict.set_item("not_found_names", not_found_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(filled_doc),
        },
        result_dict,
    ))
}
