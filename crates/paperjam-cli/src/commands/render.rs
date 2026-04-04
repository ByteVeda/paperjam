use paperjam_core::render::{ImageFormat, RenderOptions};

use crate::cli::{Cli, RenderArgs};
use crate::error::CliError;
use crate::util::parse_page_ranges;

pub fn run(args: &RenderArgs, cli: &Cli) -> Result<(), CliError> {
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    let pdf_bytes = std::fs::read(&args.file)?;

    // We need to know total page count for parse_page_ranges, open the doc briefly
    let doc = crate::document::open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;

    let page_nums = match &args.pages {
        Some(p) => parse_page_ranges(p, total)?,
        None => (1..=total).collect(),
    };

    let format = ImageFormat::from_str(&args.image_format);
    let options = RenderOptions {
        dpi: args.dpi,
        format,
        ..RenderOptions::default()
    };

    std::fs::create_dir_all(&args.output_dir)?;

    let images = paperjam_core::render::render_pages(&pdf_bytes, Some(&page_nums), &options, None)?;

    for img in &images {
        let filename = format!("page_{}.{}", img.page, img.format.extension());
        let path = args.output_dir.join(&filename);
        std::fs::write(&path, &img.data)?;
        if !cli.quiet {
            println!("Saved: {} ({}x{})", path.display(), img.width, img.height);
        }
    }

    if !cli.quiet {
        println!("Rendered {} page(s).", images.len());
    }

    Ok(())
}
