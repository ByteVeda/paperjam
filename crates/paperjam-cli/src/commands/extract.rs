use std::fs;

use paperjam_model::format::DocumentFormat;

use crate::cli::{
    Cli, ExtractAnnotationsArgs, ExtractBookmarksArgs, ExtractImagesArgs, ExtractMetadataArgs,
    ExtractStructureArgs, ExtractTablesArgs, ExtractTextArgs, OutputFormat,
};
use crate::document::{open_any, open_document, AnyDocument};
use crate::error::CliError;
use crate::util::parse_page_ranges;

pub fn run_text(args: &ExtractTextArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_any(&args.file, cli.password.as_deref())?;

    // For PDFs with --pages, use the per-page extraction path
    if let (AnyDocument::Pdf(ref pdf), Some(ref page_spec)) = (&doc, &args.pages) {
        let total = pdf.page_count() as u32;
        let pages = parse_page_ranges(page_spec, total)?;
        let mut texts = Vec::new();
        for &page_num in &pages {
            let page = pdf.page(page_num)?;
            let text = page.extract_text()?;
            texts.push(text);
        }
        return print_text_output(&texts, cli);
    }

    // For non-PDF or PDF without --pages, use AnyDocument
    if args.pages.is_some() && doc.format() != DocumentFormat::Pdf {
        eprintln!("Warning: --pages is only supported for PDF files; extracting all text.");
    }

    let text = doc.extract_text()?;
    print_text_output(&[text], cli)
}

fn print_text_output(texts: &[String], cli: &Cli) -> Result<(), CliError> {
    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({ "pages": texts });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            println!("{}", texts.join("\n\n"));
        }
    }
    Ok(())
}

pub fn run_tables(args: &ExtractTablesArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_any(&args.file, cli.password.as_deref())?;

    // For PDFs with --pages, use the per-page extraction path
    if let (AnyDocument::Pdf(ref pdf), Some(ref page_spec)) = (&doc, &args.pages) {
        let total = pdf.page_count() as u32;
        let pages = parse_page_ranges(page_spec, total)?;
        let opts = paperjam_core::table::TableExtractionOptions::default();

        match cli.format {
            OutputFormat::Json => {
                let mut all_tables = Vec::new();
                for &page_num in &pages {
                    let page = pdf.page(page_num)?;
                    let tables = page.extract_tables(&opts)?;
                    for table in &tables {
                        all_tables.push(serde_json::json!({
                            "page": page_num,
                            "rows": table.to_vec(),
                        }));
                    }
                }
                let json = serde_json::json!({ "tables": all_tables });
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
            OutputFormat::Text => {
                for &page_num in &pages {
                    let page = pdf.page(page_num)?;
                    let tables = page.extract_tables(&opts)?;
                    for (i, table) in tables.iter().enumerate() {
                        println!("--- Page {} / Table {} ---", page_num, i + 1);
                        for row in &table.rows {
                            let cells: Vec<&str> =
                                row.cells.iter().map(|c| c.text.as_str()).collect();
                            println!("| {} |", cells.join(" | "));
                        }
                        println!();
                    }
                }
            }
        }
        return Ok(());
    }

    if args.pages.is_some() && doc.format() != DocumentFormat::Pdf {
        eprintln!("Warning: --pages is only supported for PDF files; extracting all tables.");
    }

    // For non-PDF or PDF without --pages, use AnyDocument
    let tables = doc.extract_tables()?;

    match cli.format {
        OutputFormat::Json => {
            let all_tables: Vec<serde_json::Value> = tables
                .iter()
                .enumerate()
                .map(|(i, table)| {
                    serde_json::json!({
                        "index": i,
                        "rows": table.to_vec(),
                    })
                })
                .collect();
            let json = serde_json::json!({ "tables": all_tables });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            for (i, table) in tables.iter().enumerate() {
                println!("--- Table {} ---", i + 1);
                for row in &table.rows {
                    let cells: Vec<&str> = row.cells.iter().map(|c| c.text.as_str()).collect();
                    println!("| {} |", cells.join(" | "));
                }
                println!();
            }
        }
    }

    Ok(())
}

pub fn run_images(args: &ExtractImagesArgs, cli: &Cli) -> Result<(), CliError> {
    // Images extraction is PDF-specific for now
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;
    let pages = match &args.pages {
        Some(p) => parse_page_ranges(p, total)?,
        None => (1..=total).collect(),
    };

    fs::create_dir_all(&args.output_dir)?;

    let mut count = 0u32;
    for &page_num in &pages {
        let images = doc.extract_images(page_num)?;
        for (i, img) in images.iter().enumerate() {
            let ext = guess_image_ext(&img.filters);
            let filename = format!("page{}_img{}.{}", page_num, i + 1, ext);
            let path = args.output_dir.join(&filename);
            fs::write(&path, &img.data)?;
            if !cli.quiet {
                println!("Saved: {}", path.display());
            }
            count += 1;
        }
    }

    if !cli.quiet {
        println!("Extracted {} image(s).", count);
    }

    Ok(())
}

fn guess_image_ext(filters: &[String]) -> &str {
    for f in filters {
        if f.contains("JBIG") {
            return "jbig2";
        }
        if f.contains("JPEG") || f.contains("DCT") {
            return "jpg";
        }
        if f.contains("JPX") {
            return "jp2";
        }
    }
    "raw"
}

pub fn run_structure(args: &ExtractStructureArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_any(&args.file, cli.password.as_deref())?;
    let blocks = doc.extract_structure()?;

    match cli.format {
        OutputFormat::Json => {
            let json_blocks: Vec<serde_json::Value> = blocks
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "type": b.block_type(),
                        "text": b.text(),
                        "page": b.page(),
                    })
                })
                .collect();
            let json = serde_json::json!({ "blocks": json_blocks });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            for block in &blocks {
                println!(
                    "[{}] p{}: {}",
                    block.block_type(),
                    block.page(),
                    block.text()
                );
            }
        }
    }

    Ok(())
}

pub fn run_bookmarks(args: &ExtractBookmarksArgs, cli: &Cli) -> Result<(), CliError> {
    // Bookmarks are PDF-specific for now
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let bookmarks = doc.bookmarks()?;

    match cli.format {
        OutputFormat::Json => {
            let json_items: Vec<serde_json::Value> = bookmarks
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "title": b.title,
                        "page": b.page,
                        "level": b.level,
                    })
                })
                .collect();
            let json = serde_json::json!({ "bookmarks": json_items });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            for b in &bookmarks {
                let indent = "  ".repeat(b.level);
                println!("{}{}  (p. {})", indent, b.title, b.page);
            }
        }
    }

    Ok(())
}

pub fn run_annotations(args: &ExtractAnnotationsArgs, cli: &Cli) -> Result<(), CliError> {
    // Annotations are PDF-specific for now
    let doc = open_document(&args.file, cli.password.as_deref())?;
    let total = doc.page_count() as u32;
    let pages = match &args.pages {
        Some(p) => parse_page_ranges(p, total)?,
        None => (1..=total).collect(),
    };

    match cli.format {
        OutputFormat::Json => {
            let mut all_annots = Vec::new();
            for &page_num in &pages {
                let annots = doc.extract_annotations(page_num)?;
                for a in &annots {
                    all_annots.push(serde_json::json!({
                        "page": page_num,
                        "type": a.annotation_type.as_str(),
                        "contents": a.contents,
                        "rect": a.rect,
                    }));
                }
            }
            let json = serde_json::json!({ "annotations": all_annots });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            for &page_num in &pages {
                let annots = doc.extract_annotations(page_num)?;
                for a in &annots {
                    print!("p{} [{}]", page_num, a.annotation_type.as_str());
                    if let Some(ref c) = a.contents {
                        print!(": {}", c);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

pub fn run_metadata(args: &ExtractMetadataArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_any(&args.file, cli.password.as_deref())?;
    let meta = doc.metadata()?;
    let format = doc.format();

    match cli.format {
        OutputFormat::Json => {
            let mut json = serde_json::json!({
                "format": format.display_name(),
                "title": meta.title,
                "author": meta.author,
                "subject": meta.subject,
                "keywords": meta.keywords,
                "creator": meta.creator,
                "producer": meta.producer,
                "creation_date": meta.creation_date,
                "modification_date": meta.modification_date,
                "page_count": meta.page_count,
            });
            if format == DocumentFormat::Pdf {
                json["pdf_version"] = serde_json::json!(meta.pdf_version);
                json["is_encrypted"] = serde_json::json!(meta.is_encrypted);
                json["xmp_metadata"] = serde_json::json!(meta.xmp_metadata);
            }
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            println!("Format: {}", format.display_name());
            if let Some(ref v) = meta.title {
                println!("Title: {}", v);
            }
            if let Some(ref v) = meta.author {
                println!("Author: {}", v);
            }
            if let Some(ref v) = meta.subject {
                println!("Subject: {}", v);
            }
            if let Some(ref v) = meta.keywords {
                println!("Keywords: {}", v);
            }
            if let Some(ref v) = meta.creator {
                println!("Creator: {}", v);
            }
            if let Some(ref v) = meta.producer {
                println!("Producer: {}", v);
            }
            if let Some(ref v) = meta.creation_date {
                println!("Created: {}", v);
            }
            if let Some(ref v) = meta.modification_date {
                println!("Modified: {}", v);
            }
            if format == DocumentFormat::Pdf {
                println!("Version: {}", meta.pdf_version);
                println!("Encrypted: {}", meta.is_encrypted);
            }
            println!("Pages: {}", meta.page_count);
        }
    }

    Ok(())
}
