use crate::cli::{Cli, OutputFormat, RedactArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &RedactArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let fill_color = match &args.fill_color {
        Some(s) => Some(parse_color(s)?),
        None => None,
    };

    let (mut new_doc, result) = paperjam_core::redact::redact_text(
        &doc,
        &args.pattern,
        args.case_sensitive,
        args.regex,
        fill_color,
    )?;

    new_doc.save(&args.output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "pages_modified": result.pages_modified,
                "items_redacted": result.items_redacted,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!(
                    "Redacted {} item(s) across {} page(s) -> {}",
                    result.items_redacted,
                    result.pages_modified,
                    args.output.display()
                );
            }
        }
    }

    Ok(())
}

fn parse_color(s: &str) -> Result<[f64; 3], CliError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(CliError::InvalidArgument(format!(
            "Fill color must be r,g,b (0-255), got: {}",
            s
        )));
    }
    let mut color = [0.0f64; 3];
    for (i, part) in parts.iter().enumerate() {
        let v: u8 = part.trim().parse().map_err(|_| {
            CliError::InvalidArgument(format!("Invalid color component: {}", part.trim()))
        })?;
        color[i] = v as f64 / 255.0;
    }
    Ok(color)
}
