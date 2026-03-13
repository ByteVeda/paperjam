use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "add_watermark", signature = (document, text, font_size, rotation, opacity, color, font, position, layer, pages=None, custom_x=None, custom_y=None))]
#[allow(clippy::too_many_arguments)]
pub fn py_add_watermark(
    py: Python<'_>,
    document: &PyDocument,
    text: String,
    font_size: f64,
    rotation: f64,
    opacity: f64,
    color: Vec<f64>,
    font: String,
    position: String,
    layer: String,
    pages: Option<Vec<u32>>,
    custom_x: Option<f64>,
    custom_y: Option<f64>,
) -> PyResult<PyDocument> {
    let inner_clone = document.inner.inner().clone();
    let mut doc =
        paperjam_core::document::Document::from_lopdf(inner_clone).map_err(to_py_err)?;

    let color_arr = {
        let mut arr = [0.5f64; 3];
        for (i, v) in color.iter().take(3).enumerate() {
            arr[i] = *v;
        }
        arr
    };

    let pos = if let (Some(x), Some(y)) = (custom_x, custom_y) {
        paperjam_core::watermark::WatermarkPosition::Custom { x, y }
    } else {
        paperjam_core::watermark::WatermarkPosition::from_str(&position)
    };

    let options = paperjam_core::watermark::WatermarkOptions {
        text,
        font_size,
        rotation,
        opacity,
        color: color_arr,
        font: paperjam_core::watermark::BuiltinFont::from_str(&font),
        position: pos,
        layer: paperjam_core::watermark::WatermarkLayer::from_str(&layer),
        pages,
    };

    py.allow_threads(move || {
        paperjam_core::watermark::add_watermark(&mut doc, &options)?;
        Ok::<_, paperjam_core::error::PdfError>(doc)
    })
    .map_err(to_py_err)
    .map(|doc| PyDocument {
        inner: std::sync::Arc::new(doc),
    })
}
