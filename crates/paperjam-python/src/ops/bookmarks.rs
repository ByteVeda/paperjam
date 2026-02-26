use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "set_bookmarks")]
pub fn py_set_bookmarks<'py>(
    py: Python<'py>,
    document: &PyDocument,
    bookmarks: Vec<Bound<'py, PyDict>>,
) -> PyResult<PyDocument> {
    let specs = parse_bookmark_list(&bookmarks)?;
    let inner = std::sync::Arc::clone(&document.inner);

    let result = py
        .allow_threads(move || paperjam_core::bookmarks::set_bookmarks(&inner, &specs))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}

fn parse_bookmark_list(
    dicts: &[Bound<'_, PyDict>],
) -> PyResult<Vec<paperjam_core::bookmarks::BookmarkSpec>> {
    let mut specs = Vec::with_capacity(dicts.len());
    for dict in dicts {
        specs.push(parse_bookmark_dict(dict)?);
    }
    Ok(specs)
}

fn parse_bookmark_dict(
    dict: &Bound<'_, PyDict>,
) -> PyResult<paperjam_core::bookmarks::BookmarkSpec> {
    let title: String = dict
        .get_item("title")?
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Bookmark missing 'title'"))?
        .extract()?;

    let page: u32 = dict
        .get_item("page")?
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Bookmark missing 'page'"))?
        .extract()?;

    let children = if let Some(children_obj) = dict.get_item("children")? {
        let children_list: Vec<Bound<'_, PyDict>> = children_obj.extract()?;
        parse_bookmark_list(&children_list)?
    } else {
        Vec::new()
    };

    Ok(paperjam_core::bookmarks::BookmarkSpec {
        title,
        page,
        children,
    })
}
