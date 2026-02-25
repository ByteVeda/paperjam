use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::convert::render::rendered_image_to_py;
use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "render_page", signature = (document, page_number, dpi=150.0, format="png", quality=85, library_path=None))]
pub fn py_render_page<'py>(
    py: Python<'py>,
    document: &PyDocument,
    page_number: u32,
    dpi: f32,
    format: &str,
    quality: u8,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    // Serialize document to bytes for pdfium
    let mut doc_clone = document.inner.inner().clone();
    let pdf_bytes = py
        .allow_threads(move || {
            let mut buf = Vec::new();
            doc_clone
                .save_to(&mut buf)
                .map(|_| buf)
                .map_err(paperjam_core::error::PdfError::from)
        })
        .map_err(to_py_err)?;

    let options = paperjam_core::render::RenderOptions {
        dpi,
        format: paperjam_core::render::ImageFormat::from_str(format),
        quality,
    };

    let img = py
        .allow_threads(move || {
            paperjam_core::render::render_page(
                &pdf_bytes,
                page_number,
                &options,
                library_path.as_deref(),
            )
        })
        .map_err(to_py_err)?;

    rendered_image_to_py(py, &img)
}

#[pyfunction]
#[pyo3(name = "render_pages", signature = (document, pages=None, dpi=150.0, format="png", quality=85, library_path=None))]
pub fn py_render_pages<'py>(
    py: Python<'py>,
    document: &PyDocument,
    pages: Option<Vec<u32>>,
    dpi: f32,
    format: &str,
    quality: u8,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyList>> {
    let mut doc_clone = document.inner.inner().clone();
    let pdf_bytes = py
        .allow_threads(move || {
            let mut buf = Vec::new();
            doc_clone
                .save_to(&mut buf)
                .map(|_| buf)
                .map_err(paperjam_core::error::PdfError::from)
        })
        .map_err(to_py_err)?;

    let options = paperjam_core::render::RenderOptions {
        dpi,
        format: paperjam_core::render::ImageFormat::from_str(format),
        quality,
    };

    let images = py
        .allow_threads(move || {
            paperjam_core::render::render_pages(
                &pdf_bytes,
                pages.as_deref(),
                &options,
                library_path.as_deref(),
            )
        })
        .map_err(to_py_err)?;

    let list = PyList::empty(py);
    for img in &images {
        let dict = rendered_image_to_py(py, img)?;
        list.append(dict)?;
    }
    Ok(list)
}

#[pyfunction]
#[pyo3(name = "render_pages_bytes", signature = (data, pages=None, dpi=150.0, format="png", quality=85, library_path=None))]
pub fn py_render_pages_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    pages: Option<Vec<u32>>,
    dpi: f32,
    format: &str,
    quality: u8,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyList>> {
    let data = data.to_vec();
    let options = paperjam_core::render::RenderOptions {
        dpi,
        format: paperjam_core::render::ImageFormat::from_str(format),
        quality,
    };

    let images = py
        .allow_threads(move || {
            paperjam_core::render::render_pages(
                &data,
                pages.as_deref(),
                &options,
                library_path.as_deref(),
            )
        })
        .map_err(to_py_err)?;

    let list = PyList::empty(py);
    for img in &images {
        let dict = rendered_image_to_py(py, img)?;
        list.append(dict)?;
    }
    Ok(list)
}

#[pyfunction]
#[pyo3(name = "render_file", signature = (data, page_number=1, dpi=150.0, format="png", quality=85, library_path=None))]
pub fn py_render_file<'py>(
    py: Python<'py>,
    data: &[u8],
    page_number: u32,
    dpi: f32,
    format: &str,
    quality: u8,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let data = data.to_vec();
    let options = paperjam_core::render::RenderOptions {
        dpi,
        format: paperjam_core::render::ImageFormat::from_str(format),
        quality,
    };

    let img = py
        .allow_threads(move || {
            paperjam_core::render::render_page(
                &data,
                page_number,
                &options,
                library_path.as_deref(),
            )
        })
        .map_err(to_py_err)?;

    rendered_image_to_py(py, &img)
}
