use paperjam_model::format::DocumentFormat;

use crate::cli::{
    Cli, ConvertAutoArgs, ConvertMarkdownArgs, ConvertPdfAArgs, ConvertToDocxArgs,
    ConvertToEpubArgs, ConvertToHtmlArgs, ConvertToPdfArgs, ConvertToXlsxArgs, OutputFormat,
};
use crate::document::{open_any, open_document, AnyDocument};
use crate::error::CliError;

pub fn run_markdown(args: &ConvertMarkdownArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_any(&args.file, cli.password.as_deref())?;

    // For PDFs, use the richer paperjam_core markdown with options
    if let AnyDocument::Pdf(ref pdf) = doc {
        let opts = paperjam_core::markdown::MarkdownOptions {
            include_page_numbers: args.page_numbers,
            html_tables: args.html_tables,
            structure_options: paperjam_core::structure::StructureOptions {
                layout_aware: args.layout_aware,
                ..Default::default()
            },
            ..Default::default()
        };

        let markdown = paperjam_core::markdown::document_to_markdown(pdf, &opts)?;
        println!("{}", markdown);
        return Ok(());
    }

    // For other formats, use the DocumentTrait to_markdown()
    let markdown = doc.to_markdown()?;
    println!("{}", markdown);

    Ok(())
}

pub fn run_pdf_a(args: &ConvertPdfAArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let level = paperjam_core::validation::PdfALevel::from_str(&args.level);
    let options = paperjam_core::conversion::ConversionOptions {
        level,
        force: false,
    };

    let (mut new_doc, result) = paperjam_core::conversion::convert_to_pdf_a(&doc, &options)?;
    new_doc.save(&args.output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": result.success,
                "level": result.level.as_str(),
                "actions_taken": result.actions_taken.len(),
                "remaining_issues": result.remaining_issues.len(),
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!(
                    "Converted to {} -> {}",
                    result.level.as_str(),
                    args.output.display()
                );
                println!("Actions taken: {}", result.actions_taken.len());
                if !result.remaining_issues.is_empty() {
                    println!("Remaining issues: {}", result.remaining_issues.len());
                    for issue in &result.remaining_issues {
                        println!("  [{}] {}", issue.severity.as_str(), issue.message);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn run_to_pdf(args: &ConvertToPdfArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

pub fn run_to_docx(args: &ConvertToDocxArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

pub fn run_to_xlsx(args: &ConvertToXlsxArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

pub fn run_to_html(args: &ConvertToHtmlArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

pub fn run_to_epub(args: &ConvertToEpubArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

pub fn run_auto(args: &ConvertAutoArgs, cli: &Cli) -> Result<(), CliError> {
    run_convert(&args.file, &args.output, cli)
}

fn run_convert(
    input: &std::path::Path,
    output: &std::path::Path,
    cli: &Cli,
) -> Result<(), CliError> {
    if !input.exists() {
        return Err(CliError::FileNotFound(input.to_path_buf()));
    }

    let from_format = DocumentFormat::detect(input);
    let to_format = DocumentFormat::detect(output);

    if from_format == DocumentFormat::Unknown {
        return Err(CliError::InvalidArgument(format!(
            "Cannot detect input format from: {}",
            input.display()
        )));
    }
    if to_format == DocumentFormat::Unknown {
        return Err(CliError::InvalidArgument(format!(
            "Cannot detect output format from: {}",
            output.display()
        )));
    }

    let report = paperjam_convert::convert(input, output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "from": report.from_format.display_name(),
                "to": report.to_format.display_name(),
                "content_blocks": report.content_blocks,
                "tables": report.tables,
                "images": report.images,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!(
                    "Converted {} -> {} ({})",
                    report.from_format.display_name(),
                    report.to_format.display_name(),
                    output.display()
                );
                println!(
                    "  {} content blocks, {} tables, {} images",
                    report.content_blocks, report.tables, report.images
                );
            }
        }
    }

    Ok(())
}
