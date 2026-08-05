use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::errors::PipelineError;

/// Run a document processing pipeline from a YAML definition string.
#[pyfunction]
#[pyo3(name = "run_pipeline")]
pub fn py_run_pipeline<'py>(py: Python<'py>, yaml: &str) -> PyResult<Bound<'py, PyDict>> {
    let definition = paperjam_pipeline::PipelineDefinition::from_yaml(yaml)
        .map_err(|e| PipelineError::new_err(e.to_string()))?;

    let result = py.allow_threads(|| {
        let engine = paperjam_pipeline::PipelineEngine::new(definition);
        engine
            .run()
            .map_err(|e| PipelineError::new_err(e.to_string()))
    })?;

    let dict = PyDict::new(py);
    dict.set_item("total_files", result.total_files)?;
    dict.set_item("succeeded", result.succeeded)?;
    dict.set_item("failed", result.failed)?;
    dict.set_item("skipped", result.skipped)?;

    let file_results = PyList::empty(py);
    for f in &result.file_results {
        let fd = PyDict::new(py);
        fd.set_item("path", f.path.display().to_string())?;
        fd.set_item("status", format!("{:?}", f.status))?;
        fd.set_item("error", &f.error)?;
        fd.set_item("duration_ms", f.duration.as_millis() as u64)?;
        file_results.append(fd)?;
    }
    dict.set_item("file_results", file_results)?;

    Ok(dict)
}

/// Validate a pipeline definition without running it.
#[pyfunction]
#[pyo3(name = "validate_pipeline")]
pub fn py_validate_pipeline(yaml: &str) -> PyResult<()> {
    let definition = paperjam_pipeline::PipelineDefinition::from_yaml(yaml)
        .map_err(|e| PipelineError::new_err(e.to_string()))?;

    definition
        .validate()
        .map_err(|e| PipelineError::new_err(e.to_string()))?;

    Ok(())
}
