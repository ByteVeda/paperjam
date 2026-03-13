use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "visual_diff")]
#[pyo3(signature = (document_a, document_b, bytes_a, bytes_b, *, dpi=150.0, highlight_color=None, mode="both", threshold=10, library_path=None))]
#[allow(clippy::too_many_arguments)]
pub fn py_visual_diff<'py>(
    py: Python<'py>,
    document_a: &PyDocument,
    document_b: &PyDocument,
    bytes_a: &[u8],
    bytes_b: &[u8],
    dpi: f32,
    highlight_color: Option<[u8; 4]>,
    mode: &str,
    threshold: u8,
    library_path: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let inner_a = std::sync::Arc::clone(&document_a.inner);
    let inner_b = std::sync::Arc::clone(&document_b.inner);
    let bytes_a = bytes_a.to_vec();
    let bytes_b = bytes_b.to_vec();
    let mode_enum = paperjam_core::diff::visual::VisualDiffMode::from_str(mode);
    let lib_path = library_path.map(String::from);

    let options = paperjam_core::diff::visual::VisualDiffOptions {
        dpi,
        highlight_color: highlight_color.unwrap_or([255, 0, 0, 128]),
        mode: mode_enum,
        threshold,
    };

    let result = py
        .allow_threads(move || {
            paperjam_core::diff::visual::visual_diff(
                &bytes_a,
                &bytes_b,
                &inner_a,
                &inner_b,
                &options,
                lib_path.as_deref(),
            )
        })
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("overall_similarity", result.overall_similarity)?;

    // Convert pages
    let pages_list = PyList::empty(py);
    for page in &result.pages {
        let page_dict = PyDict::new(py);
        page_dict.set_item("page", page.page)?;
        page_dict.set_item("similarity", page.similarity)?;
        page_dict.set_item("changed_pixel_count", page.changed_pixel_count)?;
        page_dict.set_item("image_a", PyBytes::new(py, &page.image_a.data))?;
        page_dict.set_item("image_a_width", page.image_a.width)?;
        page_dict.set_item("image_a_height", page.image_a.height)?;
        page_dict.set_item("image_b", PyBytes::new(py, &page.image_b.data))?;
        page_dict.set_item("image_b_width", page.image_b.width)?;
        page_dict.set_item("image_b_height", page.image_b.height)?;
        page_dict.set_item("diff_image", PyBytes::new(py, &page.diff_image.data))?;
        page_dict.set_item("diff_image_width", page.diff_image.width)?;
        page_dict.set_item("diff_image_height", page.diff_image.height)?;
        pages_list.append(page_dict)?;
    }
    dict.set_item("pages", pages_list)?;

    // Convert text_diff summary
    let summary = PyDict::new(py);
    summary.set_item("pages_changed", result.text_diff.summary.pages_changed)?;
    summary.set_item("pages_added", result.text_diff.summary.pages_added)?;
    summary.set_item("pages_removed", result.text_diff.summary.pages_removed)?;
    summary.set_item("total_additions", result.text_diff.summary.total_additions)?;
    summary.set_item("total_removals", result.text_diff.summary.total_removals)?;
    summary.set_item("total_changes", result.text_diff.summary.total_changes)?;
    dict.set_item("text_diff_summary", summary)?;

    Ok(dict)
}
