use crate::cli::{Cli, DeleteArgs};
use crate::document::open_document;
use crate::error::CliError;
use crate::util::parse_page_ranges;

pub fn run(args: &DeleteArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;
    let pages = parse_page_ranges(&args.pages, total)?;

    let mut new_doc = paperjam_core::manipulation::delete_pages(&doc, &pages)?;
    new_doc.save(&args.output)?;

    if !cli.quiet {
        println!(
            "Deleted {} page(s) -> {}",
            pages.len(),
            args.output.display()
        );
    }

    Ok(())
}
