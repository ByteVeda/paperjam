use std::path::PathBuf;

/// Event emitted during pipeline execution.
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    /// The file currently being processed.
    pub file: PathBuf,
    /// Name of the current step.
    pub step: String,
    /// Step index (0-based) within the pipeline.
    pub step_index: usize,
    /// Total number of steps.
    pub total_steps: usize,
    /// File index (0-based) among all input files.
    pub file_index: usize,
    /// Total number of input files.
    pub total_files: usize,
}

/// Callback for receiving pipeline progress updates.
pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, event: &ProgressEvent);
}

/// No-op progress callback.
pub struct NoProgress;

impl ProgressCallback for NoProgress {
    fn on_progress(&self, _event: &ProgressEvent) {}
}
