use crate::error::Result;
use crate::page::Page;
use crate::table::types::*;

/// Main entry point: extract tables from a page using the configured strategy.
pub fn extract_tables(page: &Page, options: &TableExtractionOptions) -> Result<Vec<Table>> {
    match options.strategy {
        TableStrategy::Lattice => super::lattice::extract_lattice_tables(page, options),
        TableStrategy::Stream => super::stream::extract_stream_tables(page, options),
        TableStrategy::Auto => {
            // Try lattice first; fall back to stream if no tables found.
            let lattice_tables = super::lattice::extract_lattice_tables(page, options)?;
            if !lattice_tables.is_empty() {
                Ok(lattice_tables)
            } else {
                super::stream::extract_stream_tables(page, options)
            }
        }
    }
}
