use std::sync::Arc;

use paperjam_core::error::Result;
use paperjam_core::markdown::MarkdownOptions;
use paperjam_core::page::Page;
use paperjam_core::table::{Table, TableExtractionOptions};

use crate::join_err;

pub async fn extract_text(page: Arc<Page>) -> Result<String> {
    tokio::task::spawn_blocking(move || page.extract_text())
        .await
        .map_err(join_err)?
}

pub async fn extract_tables(
    page: Arc<Page>,
    options: TableExtractionOptions,
) -> Result<Vec<Table>> {
    tokio::task::spawn_blocking(move || page.extract_tables(&options))
        .await
        .map_err(join_err)?
}

pub async fn to_markdown(page: Arc<Page>, options: MarkdownOptions) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        paperjam_core::markdown::page_to_markdown(&page, &options)
    })
    .await
    .map_err(join_err)?
}
