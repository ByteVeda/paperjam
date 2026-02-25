use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "merge")]
pub fn py_merge(
    py: Python<'_>,
    documents: Vec<PyRef<'_, PyDocument>>,
    deduplicate_resources: bool,
) -> PyResult<PyDocument> {
    let docs: paperjam_core::error::Result<Vec<paperjam_core::document::Document>> = documents
        .iter()
        .map(|d| {
            // Clone lopdf document directly (no serialize/reparse)
            let inner_clone = d.inner.inner().clone();
            paperjam_core::document::Document::from_lopdf(inner_clone)
        })
        .collect();

    let docs = docs.map_err(to_py_err)?;
    let options = paperjam_core::manipulation::MergeOptions {
        deduplicate_resources,
    };

    let merged = py
        .allow_threads(move || paperjam_core::manipulation::merge(docs, &options))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(merged),
    })
}

#[pyfunction]
#[pyo3(name = "split")]
pub fn py_split(
    py: Python<'_>,
    document: &PyDocument,
    ranges: Vec<(u32, u32)>,
) -> PyResult<Vec<PyDocument>> {
    let inner = std::sync::Arc::clone(&document.inner);
    let results = py
        .allow_threads(move || paperjam_core::manipulation::split(&inner, &ranges))
        .map_err(to_py_err)?;

    Ok(results
        .into_iter()
        .map(|doc| PyDocument {
            inner: std::sync::Arc::new(doc),
        })
        .collect())
}

#[pyfunction]
#[pyo3(name = "rotate_pages")]
pub fn py_rotate_pages(
    py: Python<'_>,
    document: &PyDocument,
    page_rotations: Vec<(u32, i32)>,
) -> PyResult<PyDocument> {
    // Clone the lopdf document directly (no serialize/reparse)
    let inner_clone = document.inner.inner().clone();
    let mut doc =
        paperjam_core::document::Document::from_lopdf(inner_clone).map_err(to_py_err)?;

    let rotations: Vec<(u32, paperjam_core::manipulation::Rotation)> = page_rotations
        .into_iter()
        .map(|(page, degrees)| {
            let rot = match degrees {
                90 => paperjam_core::manipulation::Rotation::Degrees90,
                180 => paperjam_core::manipulation::Rotation::Degrees180,
                270 => paperjam_core::manipulation::Rotation::Degrees270,
                _ => paperjam_core::manipulation::Rotation::Degrees0,
            };
            (page, rot)
        })
        .collect();

    py.allow_threads(move || {
        paperjam_core::manipulation::rotate_pages(&mut doc, &rotations)?;
        Ok::<_, paperjam_core::error::PdfError>(doc)
    })
    .map_err(to_py_err)
    .map(|doc| PyDocument {
        inner: std::sync::Arc::new(doc),
    })
}

#[pyfunction]
#[pyo3(name = "reorder_pages")]
pub fn py_reorder_pages(
    py: Python<'_>,
    document: &PyDocument,
    page_order: Vec<u32>,
) -> PyResult<PyDocument> {
    let inner = std::sync::Arc::clone(&document.inner);
    let result = py
        .allow_threads(move || paperjam_core::manipulation::reorder_pages(&inner, &page_order))
        .map_err(to_py_err)?;

    Ok(PyDocument {
        inner: std::sync::Arc::new(result),
    })
}

#[pyfunction]
#[pyo3(name = "optimize")]
pub fn py_optimize<'py>(
    py: Python<'py>,
    document: &PyDocument,
    compress_streams: bool,
    remove_unused: bool,
    remove_duplicates: bool,
    strip_metadata: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);
    let options = paperjam_core::optimization::OptimizeOptions {
        compress_streams,
        remove_unused_objects: remove_unused,
        remove_duplicates,
        strip_metadata,
    };

    let (optimized, result) = py
        .allow_threads(move || paperjam_core::optimization::optimize(&inner, &options))
        .map_err(to_py_err)?;

    let dict = PyDict::new(py);
    dict.set_item("original_size", result.original_size)?;
    dict.set_item("optimized_size", result.optimized_size)?;
    dict.set_item("objects_removed", result.objects_removed)?;
    dict.set_item("streams_compressed", result.streams_compressed)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(optimized),
        },
        dict,
    ))
}

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

#[pyfunction]
#[pyo3(name = "add_watermark", signature = (document, text, font_size, rotation, opacity, color, font, position, layer, pages=None))]
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

    let options = paperjam_core::watermark::WatermarkOptions {
        text,
        font_size,
        rotation,
        opacity,
        color: color_arr,
        font: paperjam_core::watermark::BuiltinFont::from_str(&font),
        position: paperjam_core::watermark::WatermarkPosition::from_str(&position),
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
