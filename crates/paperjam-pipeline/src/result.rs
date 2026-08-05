use std::path::PathBuf;
use std::time::Duration;

/// Result of executing a pipeline across all input files.
#[derive(Debug)]
pub struct PipelineResult {
    pub total_files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub file_results: Vec<FileResult>,
}

/// Result of executing a pipeline on a single file.
#[derive(Debug)]
pub struct FileResult {
    pub path: PathBuf,
    pub status: FileStatus,
    pub error: Option<String>,
    pub duration: Duration,
}

/// Status of a single file in the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Success,
    Failed,
    Skipped,
}
