use paperjam_core::structure::ContentBlock;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::convert::table::table_to_py;

/// Convert a Rust ContentBlock to a Python dict.
pub fn content_block_to_py<'py>(
    py: Python<'py>,
    block: &ContentBlock,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("type", block.block_type())?;
    dict.set_item("page", block.page())?;

    match block {
        ContentBlock::Heading {
            text, level, bbox, ..
        } => {
            dict.set_item("text", text)?;
            dict.set_item("level", *level)?;
            dict.set_item("bbox", (bbox.0, bbox.1, bbox.2, bbox.3))?;
        }
        ContentBlock::Paragraph { text, bbox, .. } => {
            dict.set_item("text", text)?;
            dict.set_item("bbox", (bbox.0, bbox.1, bbox.2, bbox.3))?;
        }
        ContentBlock::ListItem {
            text,
            indent_level,
            bbox,
            ..
        } => {
            dict.set_item("text", text)?;
            dict.set_item("indent_level", *indent_level)?;
            dict.set_item("bbox", (bbox.0, bbox.1, bbox.2, bbox.3))?;
        }
        ContentBlock::Table { table, .. } => {
            dict.set_item("table", table_to_py(py, table)?)?;
        }
    }

    Ok(dict)
}
