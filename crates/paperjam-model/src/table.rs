/// A detected table on a page.
#[derive(Debug, Clone)]
pub struct Table {
    /// Bounding box of the table region: (x_min, y_min, x_max, y_max).
    pub bbox: (f64, f64, f64, f64),
    /// Rows of the table (top to bottom).
    pub rows: Vec<Row>,
    /// Number of columns detected.
    pub col_count: usize,
    /// Which strategy successfully extracted this table.
    pub strategy: TableStrategy,
}

impl Table {
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.rows.get(row)?.cells.get(col)
    }

    pub fn to_vec(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|r| r.cells.iter().map(|c| c.text.clone()).collect())
            .collect()
    }
}

/// A row in a table.
#[derive(Debug, Clone)]
pub struct Row {
    pub cells: Vec<Cell>,
    pub y_min: f64,
    pub y_max: f64,
}

/// A single cell in a table.
#[derive(Debug, Clone)]
pub struct Cell {
    pub text: String,
    pub bbox: (f64, f64, f64, f64),
    pub col_span: u32,
    pub row_span: u32,
}

/// Strategy used for table extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStrategy {
    Lattice,
    Stream,
    Auto,
}

/// Configuration for table extraction.
#[derive(Debug, Clone)]
pub struct TableExtractionOptions {
    pub strategy: TableStrategy,
    pub min_rows: usize,
    pub min_cols: usize,
    pub snap_tolerance: f64,
    pub row_tolerance: f64,
    pub min_col_gap: f64,
}

impl Default for TableExtractionOptions {
    fn default() -> Self {
        Self {
            strategy: TableStrategy::Auto,
            min_rows: 2,
            min_cols: 2,
            snap_tolerance: 3.0,
            row_tolerance: 0.5,
            min_col_gap: 10.0,
        }
    }
}
