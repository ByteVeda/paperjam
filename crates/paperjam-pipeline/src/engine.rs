use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;

use crate::context::PipelineContext;
use crate::definition::PipelineDefinition;
use crate::error::{ErrorStrategy, PipelineError, Result};
use crate::executor::execute_step;
use crate::progress::{NoProgress, ProgressCallback, ProgressEvent};
use crate::result::{FileResult, FileStatus, PipelineResult};

/// The pipeline execution engine.
pub struct PipelineEngine {
    definition: PipelineDefinition,
    progress: Box<dyn ProgressCallback>,
}

impl PipelineEngine {
    /// Create a new engine from a pipeline definition.
    pub fn new(definition: PipelineDefinition) -> Self {
        Self {
            definition,
            progress: Box::new(NoProgress),
        }
    }

    /// Set a progress callback.
    pub fn with_progress(mut self, progress: Box<dyn ProgressCallback>) -> Self {
        self.progress = progress;
        self
    }

    /// Execute the pipeline.
    pub fn run(&self) -> Result<PipelineResult> {
        self.definition.validate()?;

        // Resolve input glob.
        let files = resolve_glob(&self.definition.input)?;
        let total_files = files.len();

        if total_files == 0 {
            return Err(PipelineError::NoFilesMatched(self.definition.input.clone()));
        }

        let file_results = if self.definition.parallel {
            self.run_parallel(&files)
        } else {
            self.run_sequential(&files)
        };

        let succeeded = file_results
            .iter()
            .filter(|r| r.status == FileStatus::Success)
            .count();
        let failed = file_results
            .iter()
            .filter(|r| r.status == FileStatus::Failed)
            .count();
        let skipped = file_results
            .iter()
            .filter(|r| r.status == FileStatus::Skipped)
            .count();

        Ok(PipelineResult {
            total_files,
            succeeded,
            failed,
            skipped,
            file_results,
        })
    }

    fn run_sequential(&self, files: &[PathBuf]) -> Vec<FileResult> {
        let mut results = Vec::new();
        for (file_idx, path) in files.iter().enumerate() {
            let result = self.process_file(path, file_idx, files.len());
            let is_failure = result.status == FileStatus::Failed;
            results.push(result);

            if is_failure && self.definition.on_error == ErrorStrategy::FailFast {
                break;
            }
        }
        results
    }

    fn run_parallel(&self, files: &[PathBuf]) -> Vec<FileResult> {
        let total = files.len();
        files
            .par_iter()
            .enumerate()
            .map(|(idx, path)| self.process_file(path, idx, total))
            .collect()
    }

    fn process_file(
        &self,
        path: &std::path::Path,
        file_idx: usize,
        total_files: usize,
    ) -> FileResult {
        let start = Instant::now();

        let mut ctx = match PipelineContext::from_file(path) {
            Ok(ctx) => ctx,
            Err(e) => {
                return FileResult {
                    path: path.to_path_buf(),
                    status: match self.definition.on_error {
                        ErrorStrategy::Skip => FileStatus::Skipped,
                        _ => FileStatus::Failed,
                    },
                    error: Some(e.to_string()),
                    duration: start.elapsed(),
                };
            }
        };

        for (step_idx, step) in self.definition.steps.iter().enumerate() {
            self.progress.on_progress(&ProgressEvent {
                file: path.to_path_buf(),
                step: step.name().to_string(),
                step_index: step_idx,
                total_steps: self.definition.steps.len(),
                file_index: file_idx,
                total_files,
            });

            if let Err(e) = execute_step(&mut ctx, step) {
                return FileResult {
                    path: path.to_path_buf(),
                    status: match self.definition.on_error {
                        ErrorStrategy::Skip => FileStatus::Skipped,
                        _ => FileStatus::Failed,
                    },
                    error: Some(format!("step '{}': {}", step.name(), e)),
                    duration: start.elapsed(),
                };
            }
        }

        FileResult {
            path: path.to_path_buf(),
            status: FileStatus::Success,
            error: None,
            duration: start.elapsed(),
        }
    }
}

/// Resolve a glob pattern to a list of file paths.
fn resolve_glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = glob::glob(pattern)?
        .filter_map(|r| r.ok())
        .filter(|p| p.is_file())
        .collect();
    Ok(paths)
}
