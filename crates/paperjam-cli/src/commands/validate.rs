use crate::cli::{Cli, OutputFormat, ValidateArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &ValidateArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    match args.standard.to_lowercase().as_str() {
        "pdf-a" | "pdfa" => run_pdf_a(&doc, args, cli),
        "pdf-ua" | "pdfua" => run_pdf_ua(&doc, cli),
        other => Err(CliError::InvalidArgument(format!(
            "Unknown standard: {}. Use pdf-a or pdf-ua.",
            other
        ))),
    }
}

fn run_pdf_a(
    doc: &paperjam_core::document::Document,
    args: &ValidateArgs,
    cli: &Cli,
) -> Result<(), CliError> {
    let level = paperjam_core::validation::PdfALevel::from_str(&args.level);
    let report = paperjam_core::validation::validate_pdf_a(doc, level)?;

    match cli.format {
        OutputFormat::Json => {
            let issues: Vec<serde_json::Value> = report
                .issues
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "severity": i.severity.as_str(),
                        "rule": i.rule,
                        "message": i.message,
                        "page": i.page,
                    })
                })
                .collect();
            let json = serde_json::json!({
                "standard": report.level.as_str(),
                "compliant": report.is_compliant,
                "issues": issues,
                "fonts_checked": report.fonts_checked,
                "pages_checked": report.pages_checked,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if report.is_compliant {
                println!("{}: COMPLIANT", report.level.as_str());
            } else {
                println!("{}: NOT COMPLIANT", report.level.as_str());
            }
            for issue in &report.issues {
                let page_str = issue
                    .page
                    .map(|p| format!(" (p. {})", p))
                    .unwrap_or_default();
                println!(
                    "  [{}]{} {}",
                    issue.severity.as_str(),
                    page_str,
                    issue.message
                );
            }
        }
    }

    Ok(())
}

fn run_pdf_ua(doc: &paperjam_core::document::Document, cli: &Cli) -> Result<(), CliError> {
    let level = paperjam_core::validation::PdfUaLevel::Ua1;
    let report = paperjam_core::validation::validate_pdf_ua(doc, level)?;

    match cli.format {
        OutputFormat::Json => {
            let issues: Vec<serde_json::Value> = report
                .issues
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "severity": i.severity.as_str(),
                        "rule": i.rule,
                        "message": i.message,
                        "page": i.page,
                    })
                })
                .collect();
            let json = serde_json::json!({
                "standard": report.level.as_str(),
                "compliant": report.is_compliant,
                "issues": issues,
                "pages_checked": report.pages_checked,
                "structure_elements_checked": report.structure_elements_checked,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if report.is_compliant {
                println!("{}: COMPLIANT", report.level.as_str());
            } else {
                println!("{}: NOT COMPLIANT", report.level.as_str());
            }
            for issue in &report.issues {
                let page_str = issue
                    .page
                    .map(|p| format!(" (p. {})", p))
                    .unwrap_or_default();
                println!(
                    "  [{}]{} {}",
                    issue.severity.as_str(),
                    page_str,
                    issue.message
                );
            }
        }
    }

    Ok(())
}
