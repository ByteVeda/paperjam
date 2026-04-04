use std::sync::Arc;

use paperjam_model::document::DocumentTrait;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;
use paperjam_model::text::TextLine;

/// Extract text from any document implementing `DocumentTrait`.
pub async fn extract_text<D>(doc: Arc<D>) -> Result<String, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.extract_text())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}

/// Extract text lines from any document implementing `DocumentTrait`.
pub async fn extract_text_lines<D>(doc: Arc<D>) -> Result<Vec<TextLine>, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.extract_text_lines())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}

/// Extract tables from any document implementing `DocumentTrait`.
pub async fn extract_tables<D>(doc: Arc<D>) -> Result<Vec<Table>, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.extract_tables())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}

/// Extract structure from any document implementing `DocumentTrait`.
pub async fn extract_structure<D>(doc: Arc<D>) -> Result<Vec<ContentBlock>, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.extract_structure())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}

/// Extract metadata from any document implementing `DocumentTrait`.
pub async fn metadata<D>(doc: Arc<D>) -> Result<Metadata, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.metadata())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}

/// Convert to markdown from any document implementing `DocumentTrait`.
pub async fn to_markdown<D>(doc: Arc<D>) -> Result<String, D::Error>
where
    D: DocumentTrait + Send + Sync + 'static,
    D::Error: Send + 'static,
{
    tokio::task::spawn_blocking(move || doc.to_markdown())
        .await
        .unwrap_or_else(|e| panic!("spawn_blocking panicked: {}", e))
}
