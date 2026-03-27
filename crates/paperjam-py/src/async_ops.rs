use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

use crate::convert::render::rendered_image_to_py;
use crate::document::PyDocument;
use crate::errors::to_py_err;
use crate::page::PyPage;

// ---------------------------------------------------------------------------
// Document async operations
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(name = "aopen")]
pub fn py_aopen<'py>(py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let doc = paperjam_async::document::open(path)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(doc),
            };
            Ok(py_doc.into_pyobject(py)?.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "aopen_with_password")]
pub fn py_aopen_with_password<'py>(
    py: Python<'py>,
    path: String,
    password: String,
) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let doc = paperjam_async::document::open_with_password(path, password)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(doc),
            };
            Ok(py_doc.into_pyobject(py)?.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "aopen_bytes")]
pub fn py_aopen_bytes<'py>(py: Python<'py>, data: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let doc = paperjam_async::document::open_bytes(data)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(doc),
            };
            Ok(py_doc.into_pyobject(py)?.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "aopen_bytes_with_password")]
pub fn py_aopen_bytes_with_password<'py>(
    py: Python<'py>,
    data: Vec<u8>,
    password: String,
) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let doc = paperjam_async::document::open_bytes_with_password(data, password)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(doc),
            };
            Ok(py_doc.into_pyobject(py)?.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "asave")]
pub fn py_asave<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
    path: String,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&document.inner);
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        paperjam_async::document::save(inner, path)
            .await
            .map_err(to_py_err)?;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pyfunction]
#[pyo3(name = "asave_bytes")]
pub fn py_asave_bytes<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&document.inner);
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let bytes = paperjam_async::document::save_bytes(inner)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| Ok(PyBytes::new(py, &bytes).into_any().unbind()))
    })
}

#[pyfunction]
#[pyo3(
    name = "ato_markdown",
    signature = (
        document,
        *,
        heading_offset=0,
        page_separator="---",
        include_page_numbers=false,
        page_number_format="<!-- page {n} -->",
        html_tables=false,
        table_header_first_row=true,
        normalize_list_markers=true,
        heading_size_ratio=1.2,
        detect_lists=true,
        include_tables=true,
        layout_aware=false,
    )
)]
#[allow(clippy::too_many_arguments)]
pub fn py_ato_markdown<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
    heading_offset: u8,
    page_separator: &'py str,
    include_page_numbers: bool,
    page_number_format: &'py str,
    html_tables: bool,
    table_header_first_row: bool,
    normalize_list_markers: bool,
    heading_size_ratio: f64,
    detect_lists: bool,
    include_tables: bool,
    layout_aware: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&document.inner);
    let options = paperjam_core::markdown::MarkdownOptions {
        heading_offset,
        page_separator: page_separator.to_string(),
        include_page_numbers,
        page_number_format: page_number_format.to_string(),
        html_tables,
        table_header_first_row,
        normalize_list_markers,
        structure_options: paperjam_core::structure::StructureOptions {
            heading_size_ratio,
            detect_lists,
            include_tables,
            layout_aware,
        },
    };
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let md = paperjam_async::document::to_markdown(inner, options)
            .await
            .map_err(to_py_err)?;
        Ok(Python::with_gil(|py| md.into_pyobject(py).unwrap().into_any().unbind()))
    })
}

#[pyfunction]
#[pyo3(
    name = "arender_page",
    signature = (document, page_number, dpi=150.0, format="png", quality=85, background_color=None, scale_to_width=None, scale_to_height=None, library_path=None)
)]
#[allow(clippy::too_many_arguments)]
pub fn py_arender_page<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
    page_number: u32,
    dpi: f32,
    format: &'py str,
    quality: u8,
    background_color: Option<Vec<u8>>,
    scale_to_width: Option<u32>,
    scale_to_height: Option<u32>,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
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

    let options = crate::ops::render::build_render_options(
        dpi,
        format,
        quality,
        background_color,
        scale_to_width,
        scale_to_height,
    );

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let img =
            paperjam_async::document::render_page(pdf_bytes, page_number, options, library_path)
                .await
                .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let dict = rendered_image_to_py(py, &img)?;
            Ok(dict.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(
    name = "arender_pages",
    signature = (document, pages=None, dpi=150.0, format="png", quality=85, background_color=None, scale_to_width=None, scale_to_height=None, library_path=None)
)]
#[allow(clippy::too_many_arguments)]
pub fn py_arender_pages<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
    pages: Option<Vec<u32>>,
    dpi: f32,
    format: &'py str,
    quality: u8,
    background_color: Option<Vec<u8>>,
    scale_to_width: Option<u32>,
    scale_to_height: Option<u32>,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
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

    let options = crate::ops::render::build_render_options(
        dpi,
        format,
        quality,
        background_color,
        scale_to_width,
        scale_to_height,
    );

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let images =
            paperjam_async::document::render_pages(pdf_bytes, pages, options, library_path)
                .await
                .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let list = PyList::empty(py);
            for img in &images {
                let dict = rendered_image_to_py(py, img)?;
                list.append(dict)?;
            }
            Ok(list.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(
    name = "arender_file",
    signature = (data, page_number=1, dpi=150.0, format="png", quality=85, background_color=None, scale_to_width=None, scale_to_height=None, library_path=None)
)]
#[allow(clippy::too_many_arguments)]
pub fn py_arender_file<'py>(
    py: Python<'py>,
    data: Vec<u8>,
    page_number: u32,
    dpi: f32,
    format: &'py str,
    quality: u8,
    background_color: Option<Vec<u8>>,
    scale_to_width: Option<u32>,
    scale_to_height: Option<u32>,
    library_path: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let options = crate::ops::render::build_render_options(
        dpi,
        format,
        quality,
        background_color,
        scale_to_width,
        scale_to_height,
    );

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let img =
            paperjam_async::document::render_page(data, page_number, options, library_path)
                .await
                .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let dict = rendered_image_to_py(py, &img)?;
            Ok(dict.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "adiff_documents")]
pub fn py_adiff_documents<'py>(
    py: Python<'py>,
    document_a: &'py PyDocument,
    document_b: &'py PyDocument,
) -> PyResult<Bound<'py, PyAny>> {
    let inner_a = Arc::clone(&document_a.inner);
    let inner_b = Arc::clone(&document_b.inner);
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let result = paperjam_async::document::diff_documents(inner_a, inner_b)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let dict = PyDict::new(py);

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

            let summary = PyDict::new(py);
            summary.set_item("pages_changed", result.summary.pages_changed)?;
            summary.set_item("pages_added", result.summary.pages_added)?;
            summary.set_item("pages_removed", result.summary.pages_removed)?;
            summary.set_item("total_additions", result.summary.total_additions)?;
            summary.set_item("total_removals", result.summary.total_removals)?;
            summary.set_item("total_changes", result.summary.total_changes)?;
            dict.set_item("summary", summary)?;

            Ok(dict.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(
    name = "aredact_text",
    signature = (document, query, case_sensitive=true, use_regex=false, fill_color=None)
)]
pub fn py_aredact_text<'py>(
    py: Python<'py>,
    document: &'py PyDocument,
    query: String,
    case_sensitive: bool,
    use_regex: bool,
    fill_color: Option<Vec<f64>>,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&document.inner);
    let color_arr = fill_color.map(|c| {
        let mut arr = [0.0f64; 3];
        for (i, v) in c.iter().take(3).enumerate() {
            arr[i] = *v;
        }
        arr
    });

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let (redacted, result) =
            paperjam_async::document::redact_text(inner, query, case_sensitive, use_regex, color_arr)
                .await
                .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(redacted),
            };

            let dict = PyDict::new(py);
            dict.set_item("pages_modified", result.pages_modified)?;
            dict.set_item("items_redacted", result.items_redacted)?;

            let items_list = PyList::empty(py);
            for item in &result.items {
                let item_dict = PyDict::new(py);
                item_dict.set_item("page", item.page)?;
                item_dict.set_item("text", &item.text)?;
                item_dict.set_item(
                    "rect",
                    (item.rect[0], item.rect[1], item.rect[2], item.rect[3]),
                )?;
                items_list.append(item_dict)?;
            }
            dict.set_item("items", items_list)?;

            let tuple = pyo3::types::PyTuple::new(
                py,
                [py_doc.into_pyobject(py)?.into_any(), dict.into_any()],
            )?;
            Ok(tuple.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(name = "amerge", signature = (documents, deduplicate_resources=false))]
pub fn py_amerge<'py>(
    py: Python<'py>,
    documents: Vec<PyRef<'py, PyDocument>>,
    deduplicate_resources: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let docs: paperjam_core::error::Result<Vec<paperjam_core::document::Document>> = documents
        .iter()
        .map(|d| {
            let inner_clone = d.inner.inner().clone();
            paperjam_core::document::Document::from_lopdf(inner_clone)
        })
        .collect();
    let docs = docs.map_err(to_py_err)?;
    let options = paperjam_core::manipulation::MergeOptions {
        deduplicate_resources,
    };

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let merged = paperjam_async::document::merge(docs, options)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let py_doc = PyDocument {
                inner: Arc::new(merged),
            };
            Ok(py_doc.into_pyobject(py)?.into_any().unbind())
        })
    })
}

// ---------------------------------------------------------------------------
// Page async operations
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(name = "apage_extract_text")]
pub fn py_apage_extract_text<'py>(
    py: Python<'py>,
    page: &'py PyPage,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&page.inner);
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let text = paperjam_async::page::extract_text(inner)
            .await
            .map_err(to_py_err)?;
        Ok(Python::with_gil(|py| {
            text.into_pyobject(py).unwrap().into_any().unbind()
        }))
    })
}

#[pyfunction]
#[pyo3(
    name = "apage_extract_tables",
    signature = (page, *, strategy="auto", min_rows=2, min_cols=2, snap_tolerance=3.0, row_tolerance=0.5, min_col_gap=10.0)
)]
#[allow(clippy::too_many_arguments)]
pub fn py_apage_extract_tables<'py>(
    py: Python<'py>,
    page: &'py PyPage,
    strategy: &'py str,
    min_rows: usize,
    min_cols: usize,
    snap_tolerance: f64,
    row_tolerance: f64,
    min_col_gap: f64,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&page.inner);
    let opts = paperjam_core::table::TableExtractionOptions {
        strategy: match strategy {
            "lattice" => paperjam_core::table::TableStrategy::Lattice,
            "stream" => paperjam_core::table::TableStrategy::Stream,
            _ => paperjam_core::table::TableStrategy::Auto,
        },
        min_rows,
        min_cols,
        snap_tolerance,
        row_tolerance,
        min_col_gap,
    };

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let tables = paperjam_async::page::extract_tables(inner, opts)
            .await
            .map_err(to_py_err)?;
        Python::with_gil(|py| {
            let list = PyList::empty(py);
            for table in &tables {
                let table_dict = crate::convert::table_to_py(py, table)?;
                list.append(table_dict)?;
            }
            Ok(list.into_any().unbind())
        })
    })
}

#[pyfunction]
#[pyo3(
    name = "apage_to_markdown",
    signature = (
        page,
        *,
        heading_offset=0,
        page_separator="---",
        include_page_numbers=false,
        page_number_format="<!-- page {n} -->",
        html_tables=false,
        table_header_first_row=true,
        normalize_list_markers=true,
        heading_size_ratio=1.2,
        detect_lists=true,
        include_tables=true,
        layout_aware=false,
    )
)]
#[allow(clippy::too_many_arguments)]
pub fn py_apage_to_markdown<'py>(
    py: Python<'py>,
    page: &'py PyPage,
    heading_offset: u8,
    page_separator: &'py str,
    include_page_numbers: bool,
    page_number_format: &'py str,
    html_tables: bool,
    table_header_first_row: bool,
    normalize_list_markers: bool,
    heading_size_ratio: f64,
    detect_lists: bool,
    include_tables: bool,
    layout_aware: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let inner = Arc::clone(&page.inner);
    let options = paperjam_core::markdown::MarkdownOptions {
        heading_offset,
        page_separator: page_separator.to_string(),
        include_page_numbers,
        page_number_format: page_number_format.to_string(),
        html_tables,
        table_header_first_row,
        normalize_list_markers,
        structure_options: paperjam_core::structure::StructureOptions {
            heading_size_ratio,
            detect_lists,
            include_tables,
            layout_aware,
        },
    };

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let md = paperjam_async::page::to_markdown(inner, options)
            .await
            .map_err(to_py_err)?;
        Ok(Python::with_gil(|py| {
            md.into_pyobject(py).unwrap().into_any().unbind()
        }))
    })
}
