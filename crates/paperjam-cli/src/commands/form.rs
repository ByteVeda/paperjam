use std::collections::HashMap;

use crate::cli::{Cli, FormFillArgs, FormInspectArgs, OutputFormat};
use crate::document::open_document;
use crate::error::CliError;

pub fn run_inspect(args: &FormInspectArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let fields = paperjam_core::forms::extract_form_fields(doc.inner())?;

    match cli.format {
        OutputFormat::Json => {
            let json_fields: Vec<serde_json::Value> = fields
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "name": f.name,
                        "type": f.field_type.as_str(),
                        "value": f.value,
                        "page": f.page,
                        "read_only": f.read_only,
                        "required": f.required,
                    })
                })
                .collect();
            let json = serde_json::json!({ "fields": json_fields });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if fields.is_empty() {
                println!("No form fields found.");
            } else {
                println!("{} form field(s):", fields.len());
                for f in &fields {
                    let flags = {
                        let mut parts = Vec::new();
                        if f.read_only {
                            parts.push("read-only");
                        }
                        if f.required {
                            parts.push("required");
                        }
                        if parts.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", parts.join(", "))
                        }
                    };
                    let val = f
                        .value
                        .as_deref()
                        .map(|v| format!(" = \"{}\"", v))
                        .unwrap_or_default();
                    println!("  {} ({}){}{}", f.name, f.field_type.as_str(), val, flags);
                }
            }
        }
    }

    Ok(())
}

pub fn run_fill(args: &FormFillArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let mut values = HashMap::new();
    for pair in &args.fields {
        let (name, value) = pair.split_once('=').ok_or_else(|| {
            CliError::InvalidArgument(format!(
                "Invalid field assignment: {}. Use name=value format.",
                pair
            ))
        })?;
        values.insert(name.to_string(), value.to_string());
    }

    let options = paperjam_core::forms::types::FillFormOptions::default();
    let (mut new_doc, result) = paperjam_core::forms::fill_form_fields(&doc, &values, &options)?;
    new_doc.save(&args.output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "fields_filled": result.fields_filled,
                "fields_not_found": result.fields_not_found,
                "not_found_names": result.not_found_names,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                println!(
                    "Filled {} field(s) -> {}",
                    result.fields_filled,
                    args.output.display()
                );
                if !result.not_found_names.is_empty() {
                    println!("  Not found: {}", result.not_found_names.join(", "));
                }
            }
        }
    }

    Ok(())
}
