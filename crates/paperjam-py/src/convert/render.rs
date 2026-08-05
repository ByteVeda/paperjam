use paperjam_core::render::RenderedImage;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

/// Convert a Rust RenderedImage to a Python dict.
pub fn rendered_image_to_py<'py>(
    py: Python<'py>,
    img: &RenderedImage,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("data", PyBytes::new(py, &img.data))?;
    dict.set_item("width", img.width)?;
    dict.set_item("height", img.height)?;
    dict.set_item("format", img.format.as_str())?;
    dict.set_item("page", img.page)?;
    Ok(dict)
}
