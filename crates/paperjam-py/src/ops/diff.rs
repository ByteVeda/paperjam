use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "diff_documents")]
pub fn py_diff_documents<'py>(
    py: Python<'py>,
    document_a: &PyDocument,
    document_b: &PyDocument,
) -> PyResult<Bound<'py, PyDict>> {
    let inner_a = std::sync::Arc::clone(&document_a.inner);
    let inner_b = std::sync::Arc::clone(&document_b.inner);
    let result = py
        .allow_threads(move || paperjam_core::diff::diff_documents(&inner_a, &inner_b))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);

    // Convert page_diffs
    let page_diffs_list = PyList::empty(py);
    for pd in &result.page_diffs {
        let pd_dict = PyDict::new(py);
        pd_dict.set_item("page", pd.page)?;
        let ops_list = PyList::empty(py);
        for op in &pd.ops {
            let op_dict = PyDict::new(py);
            op_dict.set_item("kind", op.kind.as_str())?;
            op_dict.set_item("page", op.page)?;
            op_dict.set_item("text_a", op.text_a.as_deref())?;
            op_dict.set_item("text_b", op.text_b.as_deref())?;
            op_dict.set_item("bbox_a", op.bbox_a.map(|b| (b.0, b.1, b.2, b.3)))?;
            op_dict.set_item("bbox_b", op.bbox_b.map(|b| (b.0, b.1, b.2, b.3)))?;
            op_dict.set_item("line_index_a", op.line_index_a)?;
            op_dict.set_item("line_index_b", op.line_index_b)?;
            ops_list.append(op_dict)?;
        }
        pd_dict.set_item("ops", ops_list)?;
        page_diffs_list.append(pd_dict)?;
    }
    dict.set_item("page_diffs", page_diffs_list)?;

    // Summary
    let summary = PyDict::new(py);
    summary.set_item("pages_changed", result.summary.pages_changed)?;
    summary.set_item("pages_added", result.summary.pages_added)?;
    summary.set_item("pages_removed", result.summary.pages_removed)?;
    summary.set_item("total_additions", result.summary.total_additions)?;
    summary.set_item("total_removals", result.summary.total_removals)?;
    summary.set_item("total_changes", result.summary.total_changes)?;
    dict.set_item("summary", summary)?;

    Ok(dict)
}
