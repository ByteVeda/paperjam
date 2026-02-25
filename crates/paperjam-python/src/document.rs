use paperjam_core::document::Document;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::sync::Arc;

use crate::errors::to_py_err;
use crate::page::PyPage;

/// Internal Rust document, exposed to Python as _paperjam.RustDocument.
#[pyclass(name = "RustDocument")]
pub struct PyDocument {
    pub(crate) inner: Arc<Document>,
}

#[pymethods]
impl PyDocument {
    #[staticmethod]
    fn open(py: Python<'_>, path: String) -> PyResult<Self> {
        let doc = py
            .allow_threads(|| Document::open(&path))
            .map_err(to_py_err)?;
        Ok(PyDocument {
            inner: Arc::new(doc),
        })
    }

    #[staticmethod]
    fn open_with_password(py: Python<'_>, path: String, password: String) -> PyResult<Self> {
        let doc = py
            .allow_threads(|| Document::open_with_password(&path, &password))
            .map_err(to_py_err)?;
        Ok(PyDocument {
            inner: Arc::new(doc),
        })
    }

    #[staticmethod]
    fn from_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Self> {
        let data = data.to_vec();
        let doc = py
            .allow_threads(move || Document::open_bytes(&data))
            .map_err(to_py_err)?;
        Ok(PyDocument {
            inner: Arc::new(doc),
        })
    }

    #[staticmethod]
    fn from_bytes_with_password(
        py: Python<'_>,
        data: &[u8],
        password: String,
    ) -> PyResult<Self> {
        let data = data.to_vec();
        let doc = py
            .allow_threads(move || Document::open_bytes_with_password(&data, &password))
            .map_err(to_py_err)?;
        Ok(PyDocument {
            inner: Arc::new(doc),
        })
    }

    fn page_count(&self) -> usize {
        self.inner.page_count()
    }

    fn page(&self, py: Python<'_>, number: u32) -> PyResult<PyPage> {
        let inner = Arc::clone(&self.inner);
        let page = py
            .allow_threads(move || inner.page(number))
            .map_err(to_py_err)?;
        Ok(PyPage { inner: page })
    }

    fn metadata<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let inner = Arc::clone(&self.inner);
        let meta = py
            .allow_threads(move || inner.metadata())
            .map_err(to_py_err)?;

        let dict = PyDict::new(py);
        dict.set_item("title", meta.title.as_deref())?;
        dict.set_item("author", meta.author.as_deref())?;
        dict.set_item("subject", meta.subject.as_deref())?;
        dict.set_item("keywords", meta.keywords.as_deref())?;
        dict.set_item("creator", meta.creator.as_deref())?;
        dict.set_item("producer", meta.producer.as_deref())?;
        dict.set_item("creation_date", meta.creation_date.as_deref())?;
        dict.set_item("modification_date", meta.modification_date.as_deref())?;
        dict.set_item("pdf_version", &meta.pdf_version)?;
        dict.set_item("page_count", meta.page_count)?;
        dict.set_item("is_encrypted", meta.is_encrypted)?;
        dict.set_item("xmp_metadata", meta.xmp_metadata.as_deref())?;
        Ok(dict)
    }

    fn save_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        // We need to clone the inner document to get mutable access for saving
        let mut doc_clone = self.inner.inner().clone();
        let bytes = py
            .allow_threads(move || {
                let mut buf = Vec::new();
                doc_clone
                    .save_to(&mut buf)
                    .map(|_| buf)
                    .map_err(paperjam_core::error::PdfError::from)
            })
            .map_err(to_py_err)?;

        Ok(PyBytes::new(py, &bytes))
    }

    fn save(&self, py: Python<'_>, path: String) -> PyResult<()> {
        let mut doc_clone = self.inner.inner().clone();
        py.allow_threads(move || {
            doc_clone
                .save(&path)
                .map_err(paperjam_core::error::PdfError::from)
        })
        .map_err(to_py_err)?;
        Ok(())
    }

    fn extract_images<'py>(
        &self,
        py: Python<'py>,
        page_number: u32,
    ) -> PyResult<Bound<'py, PyList>> {
        let inner = Arc::clone(&self.inner);
        let images = py
            .allow_threads(move || inner.extract_images(page_number))
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for img in &images {
            let dict = PyDict::new(py);
            dict.set_item("width", img.width)?;
            dict.set_item("height", img.height)?;
            dict.set_item("color_space", img.color_space.as_deref())?;
            dict.set_item("bits_per_component", img.bits_per_component)?;
            dict.set_item("filters", &img.filters)?;
            dict.set_item("data", PyBytes::new(py, &img.data))?;
            list.append(dict)?;
        }
        Ok(list)
    }

    fn bookmarks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let inner = Arc::clone(&self.inner);
        let items = py
            .allow_threads(move || inner.bookmarks())
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for item in &items {
            let dict = PyDict::new(py);
            dict.set_item("title", &item.title)?;
            dict.set_item("page", item.page)?;
            dict.set_item("level", item.level)?;
            list.append(dict)?;
        }
        Ok(list)
    }

    fn annotations<'py>(
        &self,
        py: Python<'py>,
        page_number: u32,
    ) -> PyResult<Bound<'py, PyList>> {
        let inner = Arc::clone(&self.inner);
        let annots = py
            .allow_threads(move || inner.extract_annotations(page_number))
            .map_err(to_py_err)?;

        let list = PyList::empty(py);
        for annot in &annots {
            let dict = PyDict::new(py);
            dict.set_item("type", annot.annotation_type.as_str())?;
            dict.set_item(
                "rect",
                annot.rect.to_vec(),
            )?;
            dict.set_item("contents", annot.contents.as_deref())?;
            dict.set_item("author", annot.author.as_deref())?;
            dict.set_item(
                "color",
                annot.color.map(|c| c.to_vec()),
            )?;
            dict.set_item("creation_date", annot.creation_date.as_deref())?;
            dict.set_item("opacity", annot.opacity)?;
            list.append(dict)?;
        }
        Ok(list)
    }
}
