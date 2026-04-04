use paperjam_model::document::DocumentTrait;
use paperjam_model::format::DocumentFormat;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

use crate::convert::structure::content_block_to_py;
use crate::convert::table::table_to_py;
use crate::errors::FormatError;

/// Holds any non-PDF document type.
#[allow(clippy::large_enum_variant)]
pub enum AnyDocumentInner {
    Docx(paperjam_docx::DocxDocument),
    Xlsx(paperjam_xlsx::XlsxDocument),
    Pptx(paperjam_pptx::PptxDocument),
    Html(paperjam_html::HtmlDocument),
    Epub(paperjam_epub::EpubDocument),
}

/// Dispatch a DocumentTrait method call to the held variant.
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $args:expr)*) => {
        match &$self.inner {
            AnyDocumentInner::Docx(d) => d.$method($($args),*).map_err(|e| to_any_err(e)),
            AnyDocumentInner::Xlsx(d) => d.$method($($args),*).map_err(|e| to_any_err(e)),
            AnyDocumentInner::Pptx(d) => d.$method($($args),*).map_err(|e| to_any_err(e)),
            AnyDocumentInner::Html(d) => d.$method($($args),*).map_err(|e| to_any_err(e)),
            AnyDocumentInner::Epub(d) => d.$method($($args),*).map_err(|e| to_any_err(e)),
        }
    };
}

fn to_any_err(e: impl std::error::Error) -> PyErr {
    FormatError::new_err(e.to_string())
}

/// A format-agnostic document (non-PDF).
#[pyclass(name = "RustAnyDocument", unsendable)]
pub struct PyAnyDocument {
    inner: AnyDocumentInner,
}

#[pymethods]
impl PyAnyDocument {
    /// Open a document from a file path with a format hint.
    #[staticmethod]
    #[pyo3(signature = (path, format=None))]
    fn open(_py: Python<'_>, path: String, format: Option<String>) -> PyResult<Self> {
        let bytes = std::fs::read(&path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        let doc_format = if let Some(f) = format {
            DocumentFormat::from_extension(&f)
        } else {
            DocumentFormat::detect(std::path::Path::new(&path))
        };

        Self::from_bytes_inner(&bytes, doc_format)
    }

    /// Open a document from bytes with an explicit format.
    #[staticmethod]
    fn from_bytes(_py: Python<'_>, data: &[u8], format: String) -> PyResult<Self> {
        let doc_format = DocumentFormat::from_extension(&format);
        Self::from_bytes_inner(data, doc_format)
    }

    /// Document format name (e.g., "docx", "html").
    fn format_name(&self) -> &str {
        match &self.inner {
            AnyDocumentInner::Docx(_) => "docx",
            AnyDocumentInner::Xlsx(_) => "xlsx",
            AnyDocumentInner::Pptx(_) => "pptx",
            AnyDocumentInner::Html(_) => "html",
            AnyDocumentInner::Epub(_) => "epub",
        }
    }

    /// Number of pages (or sheets/slides/chapters).
    fn page_count(&self) -> usize {
        match &self.inner {
            AnyDocumentInner::Docx(d) => d.page_count(),
            AnyDocumentInner::Xlsx(d) => d.page_count(),
            AnyDocumentInner::Pptx(d) => d.page_count(),
            AnyDocumentInner::Html(d) => d.page_count(),
            AnyDocumentInner::Epub(d) => d.page_count(),
        }
    }

    /// Extract metadata as a Python dict.
    fn metadata<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let meta = dispatch!(self, metadata)?;
        let dict = PyDict::new(py);
        dict.set_item("title", meta.title)?;
        dict.set_item("author", meta.author)?;
        dict.set_item("subject", meta.subject)?;
        dict.set_item("keywords", meta.keywords)?;
        dict.set_item("creator", meta.creator)?;
        dict.set_item("producer", meta.producer)?;
        dict.set_item("creation_date", meta.creation_date)?;
        dict.set_item("modification_date", meta.modification_date)?;
        dict.set_item("page_count", meta.page_count)?;
        Ok(dict)
    }

    /// Extract all text from the document.
    fn extract_text(&self, _py: Python<'_>) -> PyResult<String> {
        dispatch!(self, extract_text)
    }

    /// Extract text lines as a list of dicts.
    fn extract_text_lines<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let lines = dispatch!(self, extract_text_lines)?;
        let list = PyList::empty(py);
        for line in &lines {
            let dict = PyDict::new(py);
            dict.set_item("text", line.text())?;
            dict.set_item("bbox", line.bbox)?;
            list.append(dict)?;
        }
        Ok(list)
    }

    /// Extract tables as a list of dicts.
    fn extract_tables<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let tables = dispatch!(self, extract_tables)?;
        let list = PyList::empty(py);
        for table in &tables {
            list.append(table_to_py(py, table)?)?;
        }
        Ok(list)
    }

    /// Extract document structure as a list of dicts.
    fn extract_structure<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let blocks = dispatch!(self, extract_structure)?;
        let list = PyList::empty(py);
        for block in &blocks {
            list.append(content_block_to_py(py, block)?)?;
        }
        Ok(list)
    }

    /// Extract images as a list of dicts.
    fn extract_images<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let images = dispatch!(self, extract_images)?;
        let list = PyList::empty(py);
        for img in &images {
            let dict = PyDict::new(py);
            dict.set_item("width", img.width)?;
            dict.set_item("height", img.height)?;
            dict.set_item("color_space", &img.color_space)?;
            dict.set_item("data_length", img.data.len())?;
            list.append(dict)?;
        }
        Ok(list)
    }

    /// Get bookmarks as a list of dicts.
    fn bookmarks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let bookmarks = dispatch!(self, bookmarks)?;
        let list = PyList::empty(py);
        for bm in &bookmarks {
            let dict = PyDict::new(py);
            dict.set_item("title", &bm.title)?;
            dict.set_item("page", bm.page)?;
            dict.set_item("level", bm.level)?;
            list.append(dict)?;
        }
        Ok(list)
    }

    /// Convert the document to Markdown.
    fn to_markdown(&self, _py: Python<'_>) -> PyResult<String> {
        dispatch!(self, to_markdown)
    }

    /// Save the document to bytes.
    fn save_to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = dispatch!(self, save_to_bytes)?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Convert to another format, returning bytes.
    fn convert_to<'py>(
        &self,
        py: Python<'py>,
        target_format: String,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = dispatch!(self, save_to_bytes)?;
        let src_format = match &self.inner {
            AnyDocumentInner::Docx(_) => DocumentFormat::Docx,
            AnyDocumentInner::Xlsx(_) => DocumentFormat::Xlsx,
            AnyDocumentInner::Pptx(_) => DocumentFormat::Pptx,
            AnyDocumentInner::Html(_) => DocumentFormat::Html,
            AnyDocumentInner::Epub(_) => DocumentFormat::Epub,
        };
        let target = DocumentFormat::from_extension(&target_format);

        let output = paperjam_convert::convert_bytes(&bytes, src_format, target)
            .map_err(|e| FormatError::new_err(e.to_string()))?;

        Ok(PyBytes::new(py, &output))
    }
}

impl PyAnyDocument {
    fn from_bytes_inner(bytes: &[u8], format: DocumentFormat) -> PyResult<Self> {
        let inner = match format {
            DocumentFormat::Docx => AnyDocumentInner::Docx(
                paperjam_docx::DocxDocument::from_bytes(bytes).map_err(to_any_err)?,
            ),
            DocumentFormat::Xlsx => AnyDocumentInner::Xlsx(
                paperjam_xlsx::XlsxDocument::open_bytes(bytes).map_err(to_any_err)?,
            ),
            DocumentFormat::Pptx => AnyDocumentInner::Pptx(
                paperjam_pptx::PptxDocument::from_bytes(bytes).map_err(to_any_err)?,
            ),
            DocumentFormat::Html => AnyDocumentInner::Html(
                paperjam_html::HtmlDocument::from_bytes(bytes).map_err(to_any_err)?,
            ),
            DocumentFormat::Epub => AnyDocumentInner::Epub(
                paperjam_epub::EpubDocument::from_bytes(bytes).map_err(to_any_err)?,
            ),
            _ => {
                return Err(FormatError::new_err(format!(
                    "Unsupported format: {}. Use Document for PDF.",
                    format.display_name()
                )));
            }
        };
        Ok(Self { inner })
    }
}
