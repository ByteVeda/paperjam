use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

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
#[pyo3(name = "redact_text", signature = (document, query, case_sensitive=true, use_regex=false, fill_color=None))]
pub fn py_redact_text<'py>(
    py: Python<'py>,
    document: &PyDocument,
    query: String,
    case_sensitive: bool,
    use_regex: bool,
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
            paperjam_core::redact::redact_text(&inner, &query, case_sensitive, use_regex, color_arr)
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

#[pyfunction]
#[pyo3(name = "encrypt_document", signature = (document, user_password, owner_password=None, permissions=None))]
pub fn py_encrypt<'py>(
    py: Python<'py>,
    document: &PyDocument,
    user_password: String,
    owner_password: Option<String>,
    permissions: Option<HashMap<String, bool>>,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let owner_pw = owner_password.unwrap_or_else(|| user_password.clone());

    let perms = if let Some(map) = permissions {
        paperjam_core::encryption::Permissions {
            print: *map.get("print").unwrap_or(&true),
            modify: *map.get("modify").unwrap_or(&true),
            copy: *map.get("copy").unwrap_or(&true),
            annotate: *map.get("annotate").unwrap_or(&true),
            fill_forms: *map.get("fill_forms").unwrap_or(&true),
            accessibility: *map.get("accessibility").unwrap_or(&true),
            assemble: *map.get("assemble").unwrap_or(&true),
            print_high_quality: *map.get("print_high_quality").unwrap_or(&true),
        }
    } else {
        paperjam_core::encryption::Permissions::default()
    };

    let options = paperjam_core::encryption::EncryptionOptions {
        user_password,
        owner_password: owner_pw,
        permissions: perms,
    };

    let encrypted_bytes = py
        .allow_threads(move || paperjam_core::encryption::encrypt(&inner, &options))
        .map_err(to_py_err)?;

    let stats = PyDict::new(py);
    stats.set_item("algorithm", "RC4-128")?;
    stats.set_item("key_length", 128)?;

    Ok((PyBytes::new(py, &encrypted_bytes), stats))
}
