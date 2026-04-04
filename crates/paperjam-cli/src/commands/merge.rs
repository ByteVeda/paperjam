use crate::cli::{Cli, MergeArgs};
use crate::error::CliError;

pub fn run(args: &MergeArgs, cli: &Cli) -> Result<(), CliError> {
    if args.files.len() < 2 {
        return Err(CliError::InvalidArgument(
            "At least 2 input files are required for merging".into(),
        ));
    }

    for f in &args.files {
        if !f.exists() {
            return Err(CliError::FileNotFound(f.clone()));
        }
    }

    let options = paperjam_core::manipulation::MergeOptions::default();
    let mut doc = paperjam_core::manipulation::merge_files(&args.files, &options)?;
    doc.save(&args.output)?;

    if !cli.quiet {
        println!(
            "Merged {} files -> {}",
            args.files.len(),
            args.output.display()
        );
    }

    Ok(())
}
