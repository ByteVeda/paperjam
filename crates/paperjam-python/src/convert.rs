use paperjam_core::table::Table;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert a Rust Table to a Python dict.
pub fn table_to_py<'py>(py: Python<'py>, table: &Table) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("bbox", (table.bbox.0, table.bbox.1, table.bbox.2, table.bbox.3))?;
    dict.set_item("col_count", table.col_count)?;
    dict.set_item(
        "strategy",
        match table.strategy {
            paperjam_core::table::TableStrategy::Lattice => "lattice",
            paperjam_core::table::TableStrategy::Stream => "stream",
            paperjam_core::table::TableStrategy::Auto => "auto",
        },
    )?;

    let rows_list = PyList::empty(py);
    for row in &table.rows {
        let row_dict = PyDict::new(py);
        row_dict.set_item("y_min", row.y_min)?;
        row_dict.set_item("y_max", row.y_max)?;

        let cells_list = PyList::empty(py);
        for cell in &row.cells {
            let cell_dict = PyDict::new(py);
            cell_dict.set_item("text", &cell.text)?;
            cell_dict.set_item(
                "bbox",
                (cell.bbox.0, cell.bbox.1, cell.bbox.2, cell.bbox.3),
            )?;
            cell_dict.set_item("col_span", cell.col_span)?;
            cell_dict.set_item("row_span", cell.row_span)?;
            cells_list.append(cell_dict)?;
        }
        row_dict.set_item("cells", cells_list)?;
        rows_list.append(row_dict)?;
    }
    dict.set_item("rows", rows_list)?;

    Ok(dict)
}
