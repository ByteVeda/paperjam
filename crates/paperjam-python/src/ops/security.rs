use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "sanitize", signature = (document, remove_javascript=true, remove_embedded_files=true, remove_actions=true, remove_links=true))]
pub fn py_sanitize<'py>(
    py: Python<'py>,
    document: &PyDocument,
    remove_javascript: bool,
    remove_embedded_files: bool,
    remove_actions: bool,
    remove_links: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);
    let options = paperjam_core::sanitize::SanitizeOptions {
        remove_javascript,
        remove_embedded_files,
        remove_actions,
        remove_links,
    };

    let (sanitized, result) = py
        .allow_threads(move || paperjam_core::sanitize::sanitize(&inner, &options))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("javascript_removed", result.javascript_removed)?;
    dict.set_item("embedded_files_removed", result.embedded_files_removed)?;
    dict.set_item("actions_removed", result.actions_removed)?;
    dict.set_item("links_removed", result.links_removed)?;

    let items_list = PyList::empty(py);
    for item in &result.items {
        let item_dict = PyDict::new(py);
        item_dict.set_item("category", &item.category)?;
        item_dict.set_item("description", &item.description)?;
        item_dict.set_item("page", item.page)?;
        items_list.append(item_dict)?;
    }
    dict.set_item("items", items_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(sanitized),
        },
        dict,
    ))
}

#[pyfunction]
#[pyo3(name = "redact", signature = (document, regions, fill_color=None))]
pub fn py_redact<'py>(
    py: Python<'py>,
    document: &PyDocument,
    regions: Vec<Bound<'py, PyDict>>,
    fill_color: Option<Vec<f64>>,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let mut redact_regions = Vec::new();
    for region_dict in &regions {
        let page: u32 = region_dict
            .get_item("page")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Region missing 'page'"))?
            .extract()?;
        let rect: Vec<f64> = region_dict
            .get_item("rect")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Region missing 'rect'"))?
            .extract()?;
        if rect.len() < 4 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "rect must have 4 elements",
            ));
        }
        redact_regions.push(paperjam_core::redact::RedactRegion {
            page,
            rect: [rect[0], rect[1], rect[2], rect[3]],
        });
    }

    let color_arr = fill_color.map(|c| {
        let mut arr = [0.0f64; 3];
        for (i, v) in c.iter().take(3).enumerate() {
            arr[i] = *v;
        }
        arr
    });

    let options = paperjam_core::redact::RedactOptions {
        regions: redact_regions,
        fill_color: color_arr,
    };

    let (redacted, result) = py
        .allow_threads(move || paperjam_core::redact::redact(&inner, &options))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("pages_modified", result.pages_modified)?;
    dict.set_item("items_redacted", result.items_redacted)?;

    let items_list = PyList::empty(py);
    for item in &result.items {
        let item_dict = PyDict::new(py);
        item_dict.set_item("page", item.page)?;
        item_dict.set_item("text", &item.text)?;
        item_dict.set_item(
            "rect",
            (item.rect[0], item.rect[1], item.rect[2], item.rect[3]),
        )?;
        items_list.append(item_dict)?;
    }
    dict.set_item("items", items_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(redacted),
        },
        dict,
    ))
}

#[pyfunction]
#[pyo3(name = "redact_text", signature = (document, query, case_sensitive=true, fill_color=None))]
pub fn py_redact_text<'py>(
    py: Python<'py>,
    document: &PyDocument,
    query: String,
    case_sensitive: bool,
    fill_color: Option<Vec<f64>>,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let color_arr = fill_color.map(|c| {
        let mut arr = [0.0f64; 3];
        for (i, v) in c.iter().take(3).enumerate() {
            arr[i] = *v;
        }
        arr
    });

    let (redacted, result) = py
        .allow_threads(move || {
            paperjam_core::redact::redact_text(&inner, &query, case_sensitive, color_arr)
        })
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("pages_modified", result.pages_modified)?;
    dict.set_item("items_redacted", result.items_redacted)?;

    let items_list = PyList::empty(py);
    for item in &result.items {
        let item_dict = PyDict::new(py);
        item_dict.set_item("page", item.page)?;
        item_dict.set_item("text", &item.text)?;
        item_dict.set_item(
            "rect",
            (item.rect[0], item.rect[1], item.rect[2], item.rect[3]),
        )?;
        items_list.append(item_dict)?;
    }
    dict.set_item("items", items_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(redacted),
        },
        dict,
    ))
}
