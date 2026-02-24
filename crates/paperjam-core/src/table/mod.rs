pub mod detector;
pub mod grid;
pub mod lattice;
pub mod stream;
pub mod types;

pub use types::{Cell, Row, Table, TableExtractionOptions, TableStrategy};

use crate::error::Result;
use crate::page::Page;

/// Main entry point: extract tables from a page.
pub fn extract_tables(page: &Page, options: &TableExtractionOptions) -> Result<Vec<Table>> {
    detector::extract_tables(page, options)
}
