use paperjam_core::optimization::OptimizeOptions;

use crate::cli::{Cli, OptimizeArgs, OutputFormat};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &OptimizeArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let options = OptimizeOptions {
        compress_streams: !args.no_compress,
        strip_metadata: args.strip_metadata,
        ..OptimizeOptions::default()
    };

    let (mut new_doc, result) = paperjam_core::optimization::optimize(&doc, &options)?;
    new_doc.save(&args.output)?;

    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "original_size": result.original_size,
                "optimized_size": result.optimized_size,
                "objects_removed": result.objects_removed,
                "streams_compressed": result.streams_compressed,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if !cli.quiet {
                let saved = result.original_size as i64 - result.optimized_size as i64;
                let pct = if result.original_size > 0 {
                    (saved as f64 / result.original_size as f64) * 100.0
                } else {
                    0.0
                };
                println!("Optimized -> {}", args.output.display());
                println!(
                    "  {} -> {} ({:+.1}%, {} bytes saved)",
                    format_size(result.original_size),
                    format_size(result.optimized_size),
                    pct,
                    saved,
                );
                println!("  Objects removed: {}", result.objects_removed);
                println!("  Streams compressed: {}", result.streams_compressed);
            }
        }
    }

    Ok(())
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
