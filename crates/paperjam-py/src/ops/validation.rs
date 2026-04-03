use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "validate_pdf_a")]
#[pyo3(signature = (document, *, level="1b"))]
pub fn py_validate_pdf_a<'py>(
    py: Python<'py>,
    document: &PyDocument,
    level: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let pdf_a_level = paperjam_core::validation::PdfALevel::from_str(level);

    let report = py
        .allow_threads(move || paperjam_core::validation::validate_pdf_a(&inner, pdf_a_level))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("level", report.level.as_str())?;
    dict.set_item("is_compliant", report.is_compliant)?;
    dict.set_item("fonts_checked", report.fonts_checked)?;
    dict.set_item("pages_checked", report.pages_checked)?;

    let issues_list = PyList::empty(py);
    for issue in &report.issues {
        let issue_dict = PyDict::new(py);
        issue_dict.set_item("severity", issue.severity.as_str())?;
        issue_dict.set_item("rule", &issue.rule)?;
        issue_dict.set_item("message", &issue.message)?;
        issue_dict.set_item("page", issue.page)?;
        issues_list.append(issue_dict)?;
    }
    dict.set_item("issues", issues_list)?;

    Ok(dict)
}

#[pyfunction]
#[pyo3(name = "validate_pdf_ua")]
#[pyo3(signature = (document, *, level="1"))]
pub fn py_validate_pdf_ua<'py>(
    py: Python<'py>,
    document: &PyDocument,
    level: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let pdf_ua_level = paperjam_core::validation::PdfUaLevel::from_str(level);

    let report = py
        .allow_threads(move || paperjam_core::validation::validate_pdf_ua(&inner, pdf_ua_level))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("level", report.level.as_str())?;
    dict.set_item("is_compliant", report.is_compliant)?;
    dict.set_item("pages_checked", report.pages_checked)?;
    dict.set_item(
        "structure_elements_checked",
        report.structure_elements_checked,
    )?;

    let issues_list = PyList::empty(py);
    for issue in &report.issues {
        let issue_dict = PyDict::new(py);
        issue_dict.set_item("severity", issue.severity.as_str())?;
        issue_dict.set_item("rule", &issue.rule)?;
        issue_dict.set_item("message", &issue.message)?;
        issue_dict.set_item("page", issue.page)?;
        issues_list.append(issue_dict)?;
    }
    dict.set_item("issues", issues_list)?;

    Ok(dict)
}
