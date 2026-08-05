use crate::definition::PipelineDefinition;
use crate::error::ErrorStrategy;
use crate::step::Step;

/// Fluent builder for constructing pipeline definitions.
pub struct PipelineBuilder {
    name: Option<String>,
    input: Option<String>,
    steps: Vec<Step>,
    parallel: bool,
    on_error: ErrorStrategy,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            input: None,
            steps: Vec::new(),
            parallel: false,
            on_error: ErrorStrategy::FailFast,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn input(mut self, glob_pattern: &str) -> Self {
        self.input = Some(glob_pattern.to_string());
        self
    }

    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    pub fn on_error(mut self, strategy: ErrorStrategy) -> Self {
        self.on_error = strategy;
        self
    }

    // --- Step builders ---

    pub fn extract_text(mut self) -> Self {
        self.steps.push(Step::ExtractText { pages: None });
        self
    }

    pub fn extract_tables(mut self) -> Self {
        self.steps.push(Step::ExtractTables { strategy: None });
        self
    }

    pub fn extract_structure(mut self) -> Self {
        self.steps.push(Step::ExtractStructure);
        self
    }

    pub fn convert(mut self, format: &str) -> Self {
        self.steps.push(Step::Convert {
            format: format.to_string(),
        });
        self
    }

    pub fn to_markdown(mut self) -> Self {
        self.steps.push(Step::ToMarkdown);
        self
    }

    pub fn redact(mut self, pattern: &str) -> Self {
        self.steps.push(Step::Redact {
            pattern: pattern.to_string(),
            case_sensitive: None,
        });
        self
    }

    pub fn watermark(mut self, text: &str) -> Self {
        self.steps.push(Step::Watermark {
            text: text.to_string(),
            font_size: None,
            opacity: None,
            rotation: None,
        });
        self
    }

    pub fn optimize(mut self) -> Self {
        self.steps.push(Step::Optimize {
            strip_metadata: None,
        });
        self
    }

    pub fn sanitize(mut self) -> Self {
        self.steps.push(Step::Sanitize {
            remove_javascript: None,
            remove_embedded_files: None,
        });
        self
    }

    pub fn encrypt(mut self, password: &str) -> Self {
        self.steps.push(Step::Encrypt {
            user_password: password.to_string(),
            owner_password: None,
            algorithm: None,
        });
        self
    }

    pub fn save(mut self, path: &str) -> Self {
        self.steps.push(Step::Save {
            path: path.to_string(),
        });
        self
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    /// Build the pipeline definition.
    pub fn build(self) -> PipelineDefinition {
        PipelineDefinition {
            name: self.name,
            input: self.input.unwrap_or_default(),
            steps: self.steps,
            parallel: self.parallel,
            on_error: self.on_error,
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
