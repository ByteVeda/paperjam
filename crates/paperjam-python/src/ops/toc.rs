use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "generate_toc")]
#[pyo3(signature = (document, *, max_depth=6, heading_size_ratio=1.2, layout_aware=false, replace_existing=true))]
pub fn py_generate_toc<'py>(
    py: Python<'py>,
    document: &PyDocument,
    max_depth: u8,
    heading_size_ratio: f64,
    layout_aware: bool,
    replace_existing: bool,
) -> PyResult<(PyDocument, Bound<'py, PyList>)> {
    let inner = std::sync::Arc::clone(&document.inner);
    let options = paperjam_core::toc::TocOptions {
        max_depth,
        heading_size_ratio,
        layout_aware,
        replace_existing,
    };

    let (new_doc, specs) = py
        .allow_threads(move || paperjam_core::toc::generate_toc(&inner, &options))
        .map_err(to_py_err)?;

    let specs_list = PyList::empty(py);
    for spec in &specs {
        let dict = bookmark_spec_to_py(py, spec)?;
        specs_list.append(dict)?;
    }

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(new_doc),
        },
        specs_list,
    ))
}

fn bookmark_spec_to_py<'py>(
    py: Python<'py>,
    spec: &paperjam_core::bookmarks::BookmarkSpec,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("title", &spec.title)?;
    dict.set_item("page", spec.page)?;
    let children = PyList::empty(py);
    for child in &spec.children {
        children.append(bookmark_spec_to_py(py, child)?)?;
    }
    dict.set_item("children", children)?;
    Ok(dict)
}
