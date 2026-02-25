use pyo3::prelude::*;

mod convert;
mod document;
mod errors;
mod manipulation;
mod page;

/// The native Rust extension module for paperjam.
///
/// Imported as `paperjam._paperjam` and wrapped by the pure Python layer.
#[pymodule]
fn _paperjam(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<document::PyDocument>()?;
    m.add_class::<page::PyPage>()?;

    errors::register_exceptions(m)?;

    m.add_function(wrap_pyfunction!(manipulation::py_merge, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_split, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_rotate_pages, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_reorder_pages, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_optimize, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_add_annotation, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_remove_annotations, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_add_watermark, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_diff_documents, m)?)?;
    m.add_function(wrap_pyfunction!(manipulation::py_sanitize, m)?)?;

    Ok(())
}
