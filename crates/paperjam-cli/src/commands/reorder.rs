use crate::cli::{Cli, ReorderArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &ReorderArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let order: Result<Vec<u32>, _> = args
        .order
        .split(',')
        .map(|s| {
            s.trim().parse::<u32>().map_err(|_| {
                CliError::InvalidArgument(format!("Invalid page number: {}", s.trim()))
            })
        })
        .collect();
    let order = order?;

    let mut new_doc = paperjam_core::manipulation::reorder_pages(&doc, &order)?;
    new_doc.save(&args.output)?;

    if !cli.quiet {
        println!("Reordered pages -> {}", args.output.display());
    }

    Ok(())
}
