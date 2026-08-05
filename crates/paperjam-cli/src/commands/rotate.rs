use paperjam_core::manipulation::Rotation;

use crate::cli::{Cli, RotateArgs};
use crate::document::open_document;
use crate::error::CliError;
use crate::util::parse_page_ranges;

pub fn run(args: &RotateArgs, cli: &Cli) -> Result<(), CliError> {
    let rotation = match args.angle {
        90 => Rotation::Degrees90,
        180 => Rotation::Degrees180,
        270 => Rotation::Degrees270,
        other => {
            return Err(CliError::InvalidArgument(format!(
                "Invalid rotation angle: {}. Must be 90, 180, or 270.",
                other
            )));
        }
    };

    let mut doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;

    let pages = match &args.pages {
        Some(p) => parse_page_ranges(p, total)?,
        None => (1..=total).collect(),
    };

    let rotations: Vec<(u32, Rotation)> = pages.into_iter().map(|p| (p, rotation)).collect();
    paperjam_core::manipulation::rotate_pages(&mut doc, &rotations)?;
    doc.save(&args.output)?;

    if !cli.quiet {
        println!(
            "Rotated {} degrees -> {}",
            args.angle,
            args.output.display()
        );
    }

    Ok(())
}
