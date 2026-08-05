//! PyO3 bindings that expose paperjam's Rust engine to Python as the
//! `_paperjam` native extension module.
//!
//! The Python package (`py_src/paperjam/`) wraps these raw bindings with
//! a more idiomatic API. Every PyO3-exposed symbol registered here must
//! also appear in `py_src/paperjam/_paperjam.pyi` so static type checkers
//! can see the extension's surface.

use pyo3::prelude::*;

#[cfg(feature = "formats")]
mod any_document;
#[cfg(feature = "async")]
mod async_ops;
mod convert;
#[cfg(feature = "formats")]
mod convert_ops;
mod document;
mod errors;
mod ops;
mod page;
#[cfg(feature = "pipeline")]
mod pipeline_ops;

/// The native Rust extension module for paperjam.
///
/// Imported as `paperjam._paperjam` and wrapped by the pure Python layer.
#[pymodule]
fn _paperjam(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // PDF classes (always available).
    m.add_class::<document::PyDocument>()?;
    m.add_class::<page::PyPage>()?;

    // Format-agnostic document class.
    #[cfg(feature = "formats")]
    m.add_class::<any_document::PyAnyDocument>()?;

    errors::register_exceptions(m)?;

    // PDF operation functions.
    m.add_function(wrap_pyfunction!(ops::py_merge, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_split, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_rotate_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_reorder_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_optimize, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_add_annotation, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_remove_annotations, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_add_watermark, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_diff_documents, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_sanitize, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_redact, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_redact_text, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_fill_form, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_modify_form_field, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_add_form_field, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_render_page, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_render_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_render_file, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_render_pages_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_sign_document, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_extract_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_verify_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_delete_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_insert_blank_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_set_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_set_bookmarks, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_generate_toc, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_stamp_pages, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_visual_diff, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_validate_pdf_a, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_convert_to_pdf_a, m)?)?;
    m.add_function(wrap_pyfunction!(ops::py_validate_pdf_ua, m)?)?;

    // Format conversion functions.
    #[cfg(feature = "formats")]
    {
        m.add_function(wrap_pyfunction!(convert_ops::py_convert_file, m)?)?;
        m.add_function(wrap_pyfunction!(convert_ops::py_convert_bytes, m)?)?;
        m.add_function(wrap_pyfunction!(convert_ops::py_detect_format, m)?)?;
    }

    // Pipeline functions.
    #[cfg(feature = "pipeline")]
    {
        m.add_function(wrap_pyfunction!(pipeline_ops::py_run_pipeline, m)?)?;
        m.add_function(wrap_pyfunction!(pipeline_ops::py_validate_pipeline, m)?)?;
    }

    // Async functions.
    #[cfg(feature = "async")]
    {
        m.add_function(wrap_pyfunction!(async_ops::py_aopen, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_aopen_with_password, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_aopen_bytes, m)?)?;
        m.add_function(wrap_pyfunction!(
            async_ops::py_aopen_bytes_with_password,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_asave, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_asave_bytes, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_ato_markdown, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_arender_page, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_arender_pages, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_arender_file, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_adiff_documents, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_aredact_text, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_amerge, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_apage_extract_text, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_apage_extract_tables, m)?)?;
        m.add_function(wrap_pyfunction!(async_ops::py_apage_to_markdown, m)?)?;
    }

    Ok(())
}
