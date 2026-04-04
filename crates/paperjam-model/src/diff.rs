/// The type of change detected in a diff operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOpKind {
    /// Line exists only in document A (was removed in B).
    Removed,
    /// Line exists only in document B (was added in B).
    Added,
    /// Line text changed between documents.
    Changed,
}

impl DiffOpKind {
    pub fn as_str(&self) -> &str {
        match self {
            DiffOpKind::Removed => "removed",
            DiffOpKind::Added => "added",
            DiffOpKind::Changed => "changed",
        }
    }
}

/// A single diff operation describing one change.
#[derive(Debug, Clone)]
pub struct DiffOp {
    pub kind: DiffOpKind,
    pub page: u32,
    pub text_a: Option<String>,
    pub text_b: Option<String>,
    pub bbox_a: Option<(f64, f64, f64, f64)>,
    pub bbox_b: Option<(f64, f64, f64, f64)>,
    pub line_index_a: Option<usize>,
    pub line_index_b: Option<usize>,
}

/// Diff results for a single page.
#[derive(Debug, Clone)]
pub struct PageDiff {
    pub page: u32,
    pub ops: Vec<DiffOp>,
}

/// Summary statistics for the entire diff.
#[derive(Debug, Clone)]
pub struct DiffSummary {
    pub pages_changed: usize,
    pub pages_added: usize,
    pub pages_removed: usize,
    pub total_additions: usize,
    pub total_removals: usize,
    pub total_changes: usize,
}

/// Complete diff result between two documents.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub page_diffs: Vec<PageDiff>,
    pub summary: DiffSummary,
}
