use pyo3::prelude::*;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "add_annotation", signature = (document, page_number, annotation_type, rect, contents=None, author=None, color=None, opacity=None, quad_points=None, url=None))]
pub fn py_add_annotation(
    py: Python<'_>,
    document: &PyDocument,
    page_number: u32,
    annotation_type: String,
    rect: Vec<f64>,
    contents: Option<String>,
    author: Option<String>,
    color: Option<Vec<f64>>,
    opacity: Option<f64>,
    quad_points: Option<Vec<f64>>,
    url: Option<String>,
) -> PyResult<PyDocument> {
    let inner_clone = document.inner.inner().clone();
    let mut doc =
        paperjam_core::document::Document::from_lopdf(inner_clone).map_err(to_py_err)?;

    let color_arr = color.map(|c| {
        let mut arr = [0.0f64; 3];
        for (i, v) in c.iter().take(3).enumerate() {
            arr[i] = *v;
        }
        arr
    });

    let rect_arr = {
        let mut arr = [0.0f64; 4];
        for (i, v) in rect.iter().take(4).enumerate() {
            arr[i] = *v;
        }
        arr
    };

    let options = paperjam_core::annotations::AddAnnotationOptions {
        annotation_type: paperjam_core::annotations::AnnotationType::from_str(&annotation_type),
        rect: rect_arr,
        contents,
        author,
        color: color_arr,
        opacity,
        quad_points,
        url,
    };

    py.allow_threads(move || {
        doc.add_annotation(page_number, &options)?;
        Ok::<_, paperjam_core::error::PdfError>(doc)
    })
    .map_err(to_py_err)
    .map(|doc| PyDocument {
        inner: std::sync::Arc::new(doc),
    })
}

#[pyfunction]
#[pyo3(name = "remove_annotations")]
pub fn py_remove_annotations(
    py: Python<'_>,
    document: &PyDocument,
    page_number: u32,
) -> PyResult<(PyDocument, usize)> {
    let inner_clone = document.inner.inner().clone();
    let mut doc =
        paperjam_core::document::Document::from_lopdf(inner_clone).map_err(to_py_err)?;

    let count = py
        .allow_threads(move || {
            let count = doc.remove_annotations(page_number)?;
            Ok::<_, paperjam_core::error::PdfError>((doc, count))
        })
        .map_err(to_py_err)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(count.0),
        },
        count.1,
    ))
}
