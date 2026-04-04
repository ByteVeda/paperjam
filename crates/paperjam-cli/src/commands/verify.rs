use crate::cli::{Cli, OutputFormat, VerifyArgs};
use crate::error::CliError;

pub fn run(args: &VerifyArgs, cli: &Cli) -> Result<(), CliError> {
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    let raw_bytes = std::fs::read(&args.file)?;
    let doc = paperjam_core::document::Document::open(&args.file)?;

    let results = paperjam_core::signature::verify_signatures(doc.inner(), &raw_bytes)?;

    match cli.format {
        OutputFormat::Json => {
            let json_results: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "integrity_ok": r.integrity_ok,
                        "certificate_valid": r.certificate_valid,
                        "message": r.message,
                        "signer": r.signer,
                        "is_ltv": r.is_ltv,
                    })
                })
                .collect();
            let json = serde_json::json!({ "signatures": json_results });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            if results.is_empty() {
                println!("No signatures found.");
            } else {
                for r in &results {
                    let status = if r.integrity_ok && r.certificate_valid {
                        "VALID"
                    } else if r.integrity_ok {
                        "INTEGRITY OK (cert issue)"
                    } else {
                        "INVALID"
                    };
                    println!("[{}] {}", status, r.name);
                    if let Some(ref signer) = r.signer {
                        println!("  Signer: {}", signer);
                    }
                    println!("  {}", r.message);
                    if r.is_ltv {
                        println!("  LTV enabled");
                    }
                }
            }
        }
    }

    Ok(())
}
