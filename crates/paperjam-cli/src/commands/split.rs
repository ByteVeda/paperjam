use crate::cli::{Cli, SplitArgs};
use crate::document::open_document;
use crate::error::CliError;
use crate::util::parse_split_ranges;

pub fn run(args: &SplitArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;
    let ranges = parse_split_ranges(&args.ranges, total)?;

    let parts = paperjam_core::manipulation::split(&doc, &ranges)?;

    std::fs::create_dir_all(&args.output_dir)?;

    let stem = args
        .file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("part");

    for (i, mut part) in parts.into_iter().enumerate() {
        let (start, end) = ranges[i];
        let filename = if start == end {
            format!("{}_p{}.pdf", stem, start)
        } else {
            format!("{}_p{}-{}.pdf", stem, start, end)
        };
        let path = args.output_dir.join(&filename);
        part.save(&path)?;
        if !cli.quiet {
            println!("Saved: {}", path.display());
        }
    }

    Ok(())
}
