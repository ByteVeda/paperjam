use paperjam_core::watermark::{WatermarkLayer, WatermarkOptions, WatermarkPosition};

use crate::cli::{Cli, WatermarkArgs};
use crate::document::open_document;
use crate::error::CliError;
use crate::util::parse_page_ranges;

pub fn run(args: &WatermarkArgs, cli: &Cli) -> Result<(), CliError> {
    let mut doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;

    let pages = match &args.pages {
        Some(p) => Some(parse_page_ranges(p, total)?),
        None => None,
    };

    let options = WatermarkOptions {
        text: args.text.clone(),
        font_size: args.font_size,
        rotation: args.rotation,
        opacity: args.opacity,
        position: WatermarkPosition::from_str(&args.position),
        layer: WatermarkLayer::from_str(&args.layer),
        pages,
        ..WatermarkOptions::default()
    };

    paperjam_core::watermark::add_watermark(&mut doc, &options)?;
    doc.save(&args.output)?;

    if !cli.quiet {
        println!("Added watermark -> {}", args.output.display());
    }

    Ok(())
}
