use crate::cli::{Cli, OutputFormat, PipelineRunArgs, PipelineValidateArgs};
use crate::error::CliError;

pub fn run(args: &PipelineRunArgs, cli: &Cli) -> Result<(), CliError> {
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    let mut definition = paperjam_pipeline::PipelineDefinition::from_file(&args.file)
        .map_err(|e| CliError::InvalidArgument(e.to_string()))?;

    // CLI overrides.
    if args.parallel {
        definition.parallel = true;
    }
    if let Some(ref strategy) = args.on_error {
        definition.on_error = match strategy.as_str() {
            "skip" => paperjam_pipeline::ErrorStrategy::Skip,
            "collect-errors" | "collect" => paperjam_pipeline::ErrorStrategy::CollectErrors,
            _ => paperjam_pipeline::ErrorStrategy::FailFast,
        };
    }

    let engine = paperjam_pipeline::PipelineEngine::new(definition);
    let result = engine
        .run()
        .map_err(|e| CliError::InvalidArgument(e.to_string()))?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "total_files": result.total_files,
                "succeeded": result.succeeded,
                "failed": result.failed,
                "skipped": result.skipped,
                "file_results": result.file_results.iter().map(|f| {
                    serde_json::json!({
                        "path": f.path.display().to_string(),
                        "status": format!("{:?}", f.status),
                        "error": f.error,
                        "duration_ms": f.duration.as_millis(),
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!(
                    "Pipeline complete: {} files ({} succeeded, {} failed, {} skipped)",
                    result.total_files, result.succeeded, result.failed, result.skipped
                );
                for f in &result.file_results {
                    let status = match f.status {
                        paperjam_pipeline::FileStatus::Success => "OK",
                        paperjam_pipeline::FileStatus::Failed => "FAIL",
                        paperjam_pipeline::FileStatus::Skipped => "SKIP",
                    };
                    print!(
                        "  [{}] {} ({:.1}ms)",
                        status,
                        f.path.display(),
                        f.duration.as_secs_f64() * 1000.0
                    );
                    if let Some(ref err) = f.error {
                        print!(" - {}", err);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

pub fn validate(args: &PipelineValidateArgs, cli: &Cli) -> Result<(), CliError> {
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    let definition = paperjam_pipeline::PipelineDefinition::from_file(&args.file)
        .map_err(|e| CliError::InvalidArgument(e.to_string()))?;

    definition
        .validate()
        .map_err(|e| CliError::InvalidArgument(e.to_string()))?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "valid": true,
                "name": definition.name,
                "input": definition.input,
                "steps": definition.steps.len(),
                "parallel": definition.parallel,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!("Pipeline definition is valid");
                if let Some(ref name) = definition.name {
                    println!("  Name: {}", name);
                }
                println!("  Input: {}", definition.input);
                println!("  Steps: {}", definition.steps.len());
                println!("  Parallel: {}", definition.parallel);
            }
        }
    }

    Ok(())
}
