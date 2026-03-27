use std::sync::Arc;

use paperjam_core::diff::DiffResult;
use paperjam_core::document::Document;
use paperjam_core::error::{PdfError, Result};
use paperjam_core::manipulation::MergeOptions;
use paperjam_core::markdown::MarkdownOptions;
use paperjam_core::redact::RedactResult;
use paperjam_core::render::{RenderedImage, RenderOptions};

use crate::join_err;

pub async fn open(path: String) -> Result<Document> {
    tokio::task::spawn_blocking(move || Document::open(&path))
        .await
        .map_err(join_err)?
}

pub async fn open_with_password(path: String, password: String) -> Result<Document> {
    tokio::task::spawn_blocking(move || Document::open_with_password(&path, &password))
        .await
        .map_err(join_err)?
}

pub async fn open_bytes(data: Vec<u8>) -> Result<Document> {
    tokio::task::spawn_blocking(move || Document::open_bytes(&data))
        .await
        .map_err(join_err)?
}

pub async fn open_bytes_with_password(data: Vec<u8>, password: String) -> Result<Document> {
    tokio::task::spawn_blocking(move || Document::open_bytes_with_password(&data, &password))
        .await
        .map_err(join_err)?
}

pub async fn save(doc: Arc<Document>, path: String) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let mut inner = doc.inner().clone();
        inner.save(&path).map_err(PdfError::from)?;
        Ok(())
    })
    .await
    .map_err(join_err)?
}

pub async fn save_bytes(doc: Arc<Document>) -> Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        let mut inner = doc.inner().clone();
        let mut buf = Vec::new();
        inner
            .save_to(&mut buf)
            .map(|_| buf)
            .map_err(PdfError::from)
    })
    .await
    .map_err(join_err)?
}

pub async fn to_markdown(doc: Arc<Document>, options: MarkdownOptions) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        paperjam_core::markdown::document_to_markdown(&doc, &options)
    })
    .await
    .map_err(join_err)?
}

pub async fn diff_documents(
    doc_a: Arc<Document>,
    doc_b: Arc<Document>,
) -> Result<DiffResult> {
    tokio::task::spawn_blocking(move || paperjam_core::diff::diff_documents(&doc_a, &doc_b))
        .await
        .map_err(join_err)?
}

pub async fn redact_text(
    doc: Arc<Document>,
    query: String,
    case_sensitive: bool,
    use_regex: bool,
    fill_color: Option<[f64; 3]>,
) -> Result<(Document, RedactResult)> {
    tokio::task::spawn_blocking(move || {
        paperjam_core::redact::redact_text(&doc, &query, case_sensitive, use_regex, fill_color)
    })
    .await
    .map_err(join_err)?
}

pub async fn merge(
    documents: Vec<Document>,
    options: MergeOptions,
) -> Result<Document> {
    tokio::task::spawn_blocking(move || paperjam_core::manipulation::merge(documents, &options))
        .await
        .map_err(join_err)?
}

pub async fn render_page(
    pdf_bytes: Vec<u8>,
    page_number: u32,
    options: RenderOptions,
    library_path: Option<String>,
) -> Result<RenderedImage> {
    tokio::task::spawn_blocking(move || {
        paperjam_core::render::render_page(
            &pdf_bytes,
            page_number,
            &options,
            library_path.as_deref(),
        )
    })
    .await
    .map_err(join_err)?
}

pub async fn render_pages(
    pdf_bytes: Vec<u8>,
    page_numbers: Option<Vec<u32>>,
    options: RenderOptions,
    library_path: Option<String>,
) -> Result<Vec<RenderedImage>> {
    tokio::task::spawn_blocking(move || {
        paperjam_core::render::render_pages(
            &pdf_bytes,
            page_numbers.as_deref(),
            &options,
            library_path.as_deref(),
        )
    })
    .await
    .map_err(join_err)?
}
