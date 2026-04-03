use pyo3::prelude::*;

#[cfg(feature = "async")]
mod async_ops;
mod convert;
mod document;
mod errors;
mod ops;
mod page;

/// The native Rust extension module for paperjam.
///
/// Imported as `paperjam._paperjam` and wrapped by the pure Python layer.
#[pymodule]
fn _paperjam(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<document::PyDocument>()?;
    m.add_class::<page::PyPage>()?;

    errors::register_exceptions(m)?;

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
