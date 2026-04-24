//! Declarative multi-step document workflows defined in YAML or JSON.
//!
//! A pipeline is a sequence of steps — open, extract, convert, redact,
//! merge, save — applied to one or more input files. The engine runs
//! steps serially or in parallel and returns a per-file summary
//! (success / failure / skipped).
//!
//! Used by the `pj pipeline` CLI subcommand and the `run_pipeline` MCP
//! tool.

pub mod builder;
pub mod context;
pub mod definition;
pub mod engine;
pub mod error;
pub mod executor;
pub mod progress;
pub mod result;
pub mod step;

pub use builder::PipelineBuilder;
pub use definition::PipelineDefinition;
pub use engine::PipelineEngine;
pub use error::{ErrorStrategy, PipelineError};
pub use result::{FileResult, FileStatus, PipelineResult};
pub use step::Step;
