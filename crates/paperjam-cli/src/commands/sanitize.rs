use paperjam_core::sanitize::SanitizeOptions;

use crate::cli::{Cli, OutputFormat, SanitizeArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &SanitizeArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let options = SanitizeOptions {
        remove_javascript: !args.keep_javascript,
        remove_embedded_files: !args.keep_embedded_files,
        remove_actions: !args.keep_actions,
        remove_links: !args.keep_links,
    };

    let (mut new_doc, result) = paperjam_core::sanitize::sanitize(&doc, &options)?;
    new_doc.save(&args.output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "javascript_removed": result.javascript_removed,
                "embedded_files_removed": result.embedded_files_removed,
                "actions_removed": result.actions_removed,
                "links_removed": result.links_removed,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                let total = result.javascript_removed
                    + result.embedded_files_removed
                    + result.actions_removed
                    + result.links_removed;
                println!(
                    "Sanitized: {} item(s) removed -> {}",
                    total,
                    args.output.display()
                );
                if result.javascript_removed > 0 {
                    println!("  JavaScript: {}", result.javascript_removed);
                }
                if result.embedded_files_removed > 0 {
                    println!("  Embedded files: {}", result.embedded_files_removed);
                }
                if result.actions_removed > 0 {
                    println!("  Actions: {}", result.actions_removed);
                }
                if result.links_removed > 0 {
                    println!("  Links: {}", result.links_removed);
                }
            }
        }
    }

    Ok(())
}
