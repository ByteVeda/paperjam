#[cfg(feature = "render")]
pub mod visual;

use crate::document::Document;
use crate::error::Result;
use crate::text::layout::TextLine;

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

/// Compare two PDF documents at the text level, returning a detailed diff.
pub fn diff_documents(doc_a: &Document, doc_b: &Document) -> Result<DiffResult> {
    let count_a = doc_a.page_count() as u32;
    let count_b = doc_b.page_count() as u32;
    let common_pages = count_a.min(count_b);

    let mut total_additions: usize = 0;
    let mut total_removals: usize = 0;
    let mut total_changes: usize = 0;
    let mut pages_changed: usize = 0;

    // Compare common pages in parallel
    let common_results = crate::parallel::par_map_pages(common_pages, |page_num| {
        let page_a = doc_a.page(page_num)?;
        let page_b = doc_b.page(page_num)?;
        let lines_a = page_a.text_lines()?;
        let lines_b = page_b.text_lines()?;
        let ops = diff_lines(&lines_a, &lines_b, page_num);
        Ok(PageDiff { page: page_num, ops })
    });
    let common_diffs = crate::parallel::collect_par_results(common_results)?;

    let mut page_diffs: Vec<PageDiff> = Vec::new();
    for pd in common_diffs {
        if !pd.ops.is_empty() {
            pages_changed += 1;
            for op in &pd.ops {
                match op.kind {
                    DiffOpKind::Added => total_additions += 1,
                    DiffOpKind::Removed => total_removals += 1,
                    DiffOpKind::Changed => total_changes += 1,
                }
            }
            page_diffs.push(pd);
        }
    }

    // Pages only in A (removed)
    let pages_removed = if count_a > common_pages {
        let extra = count_a - common_pages;
        for page_num in (common_pages + 1)..=count_a {
            let page_a = doc_a.page(page_num)?;
            let lines_a = page_a.text_lines()?;
            if !lines_a.is_empty() {
                let ops: Vec<DiffOp> = lines_a
                    .iter()
                    .enumerate()
                    .map(|(i, line)| DiffOp {
                        kind: DiffOpKind::Removed,
                        page: page_num,
                        text_a: Some(line.text()),
                        text_b: None,
                        bbox_a: Some(line.bbox),
                        bbox_b: None,
                        line_index_a: Some(i),
                        line_index_b: None,
                    })
                    .collect();
                total_removals += ops.len();
                page_diffs.push(PageDiff {
                    page: page_num,
                    ops,
                });
            }
        }
        extra as usize
    } else {
        0
    };

    // Pages only in B (added)
    let pages_added = if count_b > common_pages {
        let extra = count_b - common_pages;
        for page_num in (common_pages + 1)..=count_b {
            let page_b = doc_b.page(page_num)?;
            let lines_b = page_b.text_lines()?;
            if !lines_b.is_empty() {
                let ops: Vec<DiffOp> = lines_b
                    .iter()
                    .enumerate()
                    .map(|(i, line)| DiffOp {
                        kind: DiffOpKind::Added,
                        page: page_num,
                        text_a: None,
                        text_b: Some(line.text()),
                        bbox_a: None,
                        bbox_b: Some(line.bbox),
                        line_index_a: None,
                        line_index_b: Some(i),
                    })
                    .collect();
                total_additions += ops.len();
                page_diffs.push(PageDiff {
                    page: page_num,
                    ops,
                });
            }
        }
        extra as usize
    } else {
        0
    };

    // Sort page_diffs by page number
    page_diffs.sort_by_key(|pd| pd.page);

    Ok(DiffResult {
        page_diffs,
        summary: DiffSummary {
            pages_changed,
            pages_added,
            pages_removed,
            total_additions,
            total_removals,
            total_changes,
        },
    })
}

/// Diff two sequences of text lines using LCS (longest common subsequence).
fn diff_lines(lines_a: &[TextLine], lines_b: &[TextLine], page: u32) -> Vec<DiffOp> {
    let texts_a: Vec<String> = lines_a.iter().map(|l| l.text()).collect();
    let texts_b: Vec<String> = lines_b.iter().map(|l| l.text()).collect();

    let n = texts_a.len();
    let m = texts_b.len();

    // Build LCS DP table
    let lcs = lcs_table(&texts_a, &texts_b);

    // Backtrack to produce diff ops
    let mut raw_ops = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && texts_a[i - 1] == texts_b[j - 1] {
            // Lines match, no change
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            // Line added in B
            raw_ops.push(DiffOp {
                kind: DiffOpKind::Added,
                page,
                text_a: None,
                text_b: Some(texts_b[j - 1].clone()),
                bbox_a: None,
                bbox_b: Some(lines_b[j - 1].bbox),
                line_index_a: None,
                line_index_b: Some(j - 1),
            });
            j -= 1;
        } else if i > 0 {
            // Line removed from A
            raw_ops.push(DiffOp {
                kind: DiffOpKind::Removed,
                page,
                text_a: Some(texts_a[i - 1].clone()),
                text_b: None,
                bbox_a: Some(lines_a[i - 1].bbox),
                bbox_b: None,
                line_index_a: Some(i - 1),
                line_index_b: None,
            });
            i -= 1;
        }
    }

    // Reverse since we backtracked from the end
    raw_ops.reverse();

    // Post-process: merge consecutive Removed+Added pairs into Changed
    merge_changes(raw_ops)
}

/// Compute LCS table using dynamic programming.
fn lcs_table(a: &[String], b: &[String]) -> Vec<Vec<usize>> {
    let n = a.len();
    let m = b.len();
    let mut table = vec![vec![0usize; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if a[i - 1] == b[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    table
}

/// Merge consecutive Removed+Added pairs into Changed ops when lines are similar.
fn merge_changes(ops: Vec<DiffOp>) -> Vec<DiffOp> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < ops.len() {
        if i + 1 < ops.len()
            && ops[i].kind == DiffOpKind::Removed
            && ops[i + 1].kind == DiffOpKind::Added
        {
            // Check if the lines are similar (share a common prefix of >= 30%)
            if let (Some(ref text_a), Some(ref text_b)) = (&ops[i].text_a, &ops[i + 1].text_b) {
                let shorter_len = text_a.len().min(text_b.len());
                if shorter_len > 0 {
                    let common = text_a
                        .chars()
                        .zip(text_b.chars())
                        .take_while(|(a, b)| a == b)
                        .count();
                    if common as f64 / shorter_len as f64 >= 0.3 {
                        // Merge into Changed
                        result.push(DiffOp {
                            kind: DiffOpKind::Changed,
                            page: ops[i].page,
                            text_a: ops[i].text_a.clone(),
                            text_b: ops[i + 1].text_b.clone(),
                            bbox_a: ops[i].bbox_a,
                            bbox_b: ops[i + 1].bbox_b,
                            line_index_a: ops[i].line_index_a,
                            line_index_b: ops[i + 1].line_index_b,
                        });
                        i += 2;
                        continue;
                    }
                }
            }
        }
        result.push(ops[i].clone());
        i += 1;
    }

    result
}
