use paperjam_core::toc::TocOptions;

use crate::cli::{Cli, TocArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &TocArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let options = TocOptions {
        max_depth: args.max_depth,
        ..TocOptions::default()
    };

    let (mut new_doc, specs) = paperjam_core::toc::generate_toc(&doc, &options)?;
    new_doc.save(&args.output)?;

    if !cli.quiet {
        println!(
            "Generated TOC with {} bookmark(s) -> {}",
            specs.len(),
            args.output.display()
        );
    }

    Ok(())
}
