use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::annotations::{self, AddAnnotationOptions, Annotation};
use crate::bookmarks::{self, BookmarkItem};
use crate::error::{PdfError, Result};
use crate::image::{self, ImageInfo};
use crate::metadata::Metadata;
use crate::page::Page;

/// A PDF document with lazy page loading.
///
/// Eagerly parses the xref table, trailer, and page tree to determine
/// page count and object IDs. Individual page content is parsed lazily
/// on first access.
pub struct Document {
    /// The underlying lopdf document.
    inner: lopdf::Document,
    /// Ordered map of 1-based page number -> ObjectId.
    page_map: BTreeMap<u32, lopdf::ObjectId>,
    /// Cache of parsed pages (lazily populated).
    page_cache: Mutex<BTreeMap<u32, Arc<Page>>>,
    /// Parsed metadata (lazily populated).
    metadata_cache: Mutex<Option<Arc<Metadata>>>,
}

impl Document {
    /// Open a PDF from a filesystem path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = lopdf::Document::load(path)?;
        Self::from_lopdf(inner)
    }

    /// Open a PDF from a filesystem path with a password.
    ///
    /// Note: lopdf 0.34 does not natively support password-based decryption
    /// through load_filtered. This loads the document and relies on lopdf's
    /// built-in encryption handling.
    pub fn open_with_password<P: AsRef<Path>>(path: P, password: &str) -> Result<Self> {
        let mut inner = lopdf::Document::load(path).map_err(|e| match e {
            lopdf::Error::Decryption(_) => PdfError::PasswordRequired,
            other => PdfError::Lopdf(other),
        })?;
        if inner.is_encrypted() {
            inner.decrypt(password).map_err(|e| match e {
                lopdf::Error::Decryption(_) => PdfError::InvalidPassword,
                other => PdfError::Lopdf(other),
            })?;
        }
        Self::from_lopdf(inner)
    }

    /// Open a PDF from bytes in memory.
    pub fn open_bytes(bytes: &[u8]) -> Result<Self> {
        let inner = lopdf::Document::load_mem(bytes)?;
        Self::from_lopdf(inner)
    }

    /// Open a PDF from bytes with a password.
    pub fn open_bytes_with_password(bytes: &[u8], password: &str) -> Result<Self> {
        let mut inner = lopdf::Document::load_mem(bytes).map_err(|e| match e {
            lopdf::Error::Decryption(_) => PdfError::PasswordRequired,
            other => PdfError::Lopdf(other),
        })?;
        if inner.is_encrypted() {
            inner.decrypt(password).map_err(|e| match e {
                lopdf::Error::Decryption(_) => PdfError::InvalidPassword,
                other => PdfError::Lopdf(other),
            })?;
        }
        Self::from_lopdf(inner)
    }

    pub fn from_lopdf(inner: lopdf::Document) -> Result<Self> {
        let page_map = inner.get_pages();
        Ok(Self {
            inner,
            page_map,
            page_cache: Mutex::new(BTreeMap::new()),
            metadata_cache: Mutex::new(None),
        })
    }

    /// Total number of pages.
    pub fn page_count(&self) -> usize {
        self.page_map.len()
    }

    /// Get a specific page (1-indexed). Lazily parsed and cached.
    pub fn page(&self, number: u32) -> Result<Arc<Page>> {
        if !self.page_map.contains_key(&number) {
            return Err(PdfError::PageOutOfRange {
                page: number as usize,
                total: self.page_count(),
            });
        }

        let mut cache = self.page_cache.lock().unwrap();
        if let Some(page) = cache.get(&number) {
            return Ok(Arc::clone(page));
        }

        let object_id = self.page_map[&number];
        let page = Page::parse(&self.inner, number, object_id)?;
        let page = Arc::new(page);
        cache.insert(number, Arc::clone(&page));
        Ok(page)
    }

    /// Iterate over all pages lazily.
    pub fn pages(&self) -> PageIterator<'_> {
        PageIterator {
            doc: self,
            page_numbers: self.page_map.keys().copied().collect(),
            index: 0,
        }
    }

    /// Get document metadata.
    pub fn metadata(&self) -> Result<Arc<Metadata>> {
        let mut cache = self.metadata_cache.lock().unwrap();
        if let Some(ref meta) = *cache {
            return Ok(Arc::clone(meta));
        }
        let meta = Metadata::extract(&self.inner)?;
        let meta = Arc::new(meta);
        *cache = Some(Arc::clone(&meta));
        Ok(meta)
    }

    /// Extract images from a specific page (1-indexed).
    pub fn extract_images(&self, page_number: u32) -> Result<Vec<ImageInfo>> {
        image::extract_page_images(&self.inner, page_number, &self.page_map)
    }

    /// Extract the document's bookmark/outline tree as a flat list.
    pub fn bookmarks(&self) -> Result<Vec<BookmarkItem>> {
        bookmarks::extract_bookmarks(&self.inner)
    }

    /// Extract annotations from a specific page (1-indexed).
    pub fn extract_annotations(&self, page_number: u32) -> Result<Vec<Annotation>> {
        annotations::extract_annotations(&self.inner, page_number, &self.page_map)
    }

    /// Extract only link annotations from a specific page (1-indexed).
    pub fn extract_links(&self, page_number: u32) -> Result<Vec<Annotation>> {
        annotations::extract_links(&self.inner, page_number, &self.page_map)
    }

    /// Add an annotation to a specific page.
    pub fn add_annotation(
        &mut self,
        page_number: u32,
        options: &AddAnnotationOptions,
    ) -> Result<()> {
        let page_map = self.page_map.clone();
        annotations::add_annotation(&mut self.inner, page_number, &page_map, options)
    }

    /// Remove annotations from a specific page. Returns count removed.
    ///
    /// If `annotation_types` is `Some`, only matching types are removed.
    /// If `indices` is `Some`, only annotations at those positions are removed.
    pub fn remove_annotations(
        &mut self,
        page_number: u32,
        annotation_types: Option<&[&str]>,
        indices: Option<&[usize]>,
    ) -> Result<usize> {
        let page_map = self.page_map.clone();
        annotations::remove_annotations(
            &mut self.inner,
            page_number,
            &page_map,
            annotation_types,
            indices,
        )
    }

    /// Access the underlying lopdf Document (for manipulation operations).
    pub fn inner(&self) -> &lopdf::Document {
        &self.inner
    }

    /// Get a mutable reference to the inner document.
    pub fn inner_mut(&mut self) -> &mut lopdf::Document {
        &mut self.inner
    }

    /// Take ownership of the inner document (for save operations).
    pub fn into_inner(self) -> lopdf::Document {
        self.inner
    }

    /// Save the document to a file.
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.inner.save(path)?;
        Ok(())
    }

    /// Serialize the document to bytes.
    pub fn save_to_bytes(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.inner.save_to(&mut buf)?;
        Ok(buf)
    }
}

/// Lazy iterator over pages.
pub struct PageIterator<'a> {
    doc: &'a Document,
    page_numbers: Vec<u32>,
    index: usize,
}

impl<'a> Iterator for PageIterator<'a> {
    type Item = Result<Arc<Page>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.page_numbers.len() {
            return None;
        }
        let num = self.page_numbers[self.index];
        self.index += 1;
        Some(self.doc.page(num))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.page_numbers.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for PageIterator<'a> {}
