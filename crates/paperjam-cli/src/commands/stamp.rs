use paperjam_core::stamp::{StampLayer, StampOptions};

use crate::cli::{Cli, StampArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &StampArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let stamp_doc = open_document(&args.stamp, cli.password.as_deref())?;

    let options = StampOptions {
        source_page: args.source_page,
        target_pages: None,
        x: 0.0,
        y: 0.0,
        scale: args.scale,
        opacity: args.opacity,
        layer: StampLayer::from_str(&args.layer),
    };

    let mut new_doc = paperjam_core::stamp::stamp_pages(&doc, &stamp_doc, &options)?;
    new_doc.save(&args.output)?;

    if !cli.quiet {
        println!("Stamped -> {}", args.output.display());
    }

    Ok(())
}
