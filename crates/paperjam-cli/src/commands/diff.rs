use paperjam_core::diff::DiffOpKind;

use crate::cli::{Cli, DiffArgs, OutputFormat};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &DiffArgs, cli: &Cli) -> Result<(), CliError> {
    let doc_a = open_document(&args.file_a, cli.password.as_deref())?;
    let doc_b = open_document(&args.file_b, cli.password.as_deref())?;

    let result = paperjam_core::diff::diff_documents(&doc_a, &doc_b)?;

    match cli.format {
        OutputFormat::Json => {
            let page_diffs: Vec<serde_json::Value> = result
                .page_diffs
                .iter()
                .map(|pd| {
                    let ops: Vec<serde_json::Value> = pd
                        .ops
                        .iter()
                        .map(|op| {
                            serde_json::json!({
                                "kind": op.kind.as_str(),
                                "text_a": op.text_a,
                                "text_b": op.text_b,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "page": pd.page,
                        "ops": ops,
                    })
                })
                .collect();
            let json = serde_json::json!({
                "summary": {
                    "pages_changed": result.summary.pages_changed,
                    "pages_added": result.summary.pages_added,
                    "pages_removed": result.summary.pages_removed,
                    "total_additions": result.summary.total_additions,
                    "total_removals": result.summary.total_removals,
                    "total_changes": result.summary.total_changes,
                },
                "page_diffs": page_diffs,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            for pd in &result.page_diffs {
                println!("--- Page {} ---", pd.page);
                for op in &pd.ops {
                    match op.kind {
                        DiffOpKind::Removed => {
                            if let Some(ref text) = op.text_a {
                                println!("- {}", text);
                            }
                        }
                        DiffOpKind::Added => {
                            if let Some(ref text) = op.text_b {
                                println!("+ {}", text);
                            }
                        }
                        DiffOpKind::Changed => {
                            if let Some(ref text) = op.text_a {
                                println!("- {}", text);
                            }
                            if let Some(ref text) = op.text_b {
                                println!("+ {}", text);
                            }
                        }
                    }
                }
            }

            let s = &result.summary;
            println!(
                "\n{} page(s) changed, {} addition(s), {} removal(s), {} change(s)",
                s.pages_changed, s.total_additions, s.total_removals, s.total_changes,
            );
        }
    }

    Ok(())
}
