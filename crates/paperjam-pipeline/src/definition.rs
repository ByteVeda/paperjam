use serde::{Deserialize, Serialize};

use crate::error::{ErrorStrategy, PipelineError};
use crate::step::Step;

/// A complete pipeline definition, serializable to/from YAML and JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    /// Optional pipeline name.
    #[serde(default)]
    pub name: Option<String>,
    /// Input glob pattern (e.g., "docs/*.pdf").
    pub input: String,
    /// Ordered list of steps to execute per file.
    pub steps: Vec<Step>,
    /// Whether to process files in parallel.
    #[serde(default)]
    pub parallel: bool,
    /// How to handle errors.
    #[serde(default)]
    pub on_error: ErrorStrategy,
}

impl PipelineDefinition {
    /// Parse from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, PipelineError> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    /// Parse from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, PipelineError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Parse from a file (auto-detects YAML or JSON from extension).
    pub fn from_file(path: &std::path::Path) -> Result<Self, PipelineError> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "json" => Self::from_json(&content),
            _ => Self::from_yaml(&content),
        }
    }

    /// Serialize to YAML.
    pub fn to_yaml(&self) -> Result<String, PipelineError> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, PipelineError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Validate the definition (check for obvious issues).
    pub fn validate(&self) -> Result<(), PipelineError> {
        if self.input.is_empty() {
            return Err(PipelineError::Config("input pattern is empty".to_string()));
        }
        if self.steps.is_empty() {
            return Err(PipelineError::Config("no steps defined".to_string()));
        }
        Ok(())
    }
}
