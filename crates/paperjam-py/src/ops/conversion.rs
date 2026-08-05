use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "convert_to_pdf_a", signature = (document, *, level="1b", force=false))]
pub fn py_convert_to_pdf_a<'py>(
    py: Python<'py>,
    document: &PyDocument,
    level: &str,
    force: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let pdf_a_level = paperjam_core::validation::PdfALevel::from_str(level);

    let options = paperjam_core::conversion::ConversionOptions {
        level: pdf_a_level,
        force,
    };

    let (converted, result) = py
        .allow_threads(move || paperjam_core::conversion::convert_to_pdf_a(&inner, &options))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("level", result.level.as_str())?;
    dict.set_item("success", result.success)?;

    let actions_list = PyList::empty(py);
    for action in &result.actions_taken {
        let action_dict = PyDict::new(py);
        action_dict.set_item("category", &action.category)?;
        action_dict.set_item("description", &action.description)?;
        action_dict.set_item("page", action.page)?;
        actions_list.append(action_dict)?;
    }
    dict.set_item("actions_taken", actions_list)?;

    let issues_list = PyList::empty(py);
    for issue in &result.remaining_issues {
        let issue_dict = PyDict::new(py);
        issue_dict.set_item("severity", issue.severity.as_str())?;
        issue_dict.set_item("rule", &issue.rule)?;
        issue_dict.set_item("message", &issue.message)?;
        issue_dict.set_item("page", issue.page)?;
        issues_list.append(issue_dict)?;
    }
    dict.set_item("remaining_issues", issues_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(converted),
        },
        dict,
    ))
}
