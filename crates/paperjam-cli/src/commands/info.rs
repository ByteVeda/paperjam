use crate::cli::{Cli, InfoArgs, OutputFormat};
use crate::document::{open_any, AnyDocument};
use crate::error::CliError;

pub fn run(args: &InfoArgs, cli: &Cli) -> Result<(), CliError> {
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
            // PDF-specific fields
            if format == paperjam_model::format::DocumentFormat::Pdf {
                json["pdf_version"] = serde_json::json!(meta.pdf_version);
                json["is_encrypted"] = serde_json::json!(meta.is_encrypted);
            }
            // XLSX-specific: sheet names
            if let AnyDocument::Xlsx(ref xlsx) = doc {
                json["sheet_names"] = serde_json::json!(xlsx.sheet_names());
            }
            // PPTX-specific: slide count
            if let AnyDocument::Pptx(ref pptx) = doc {
                json["slide_count"] = serde_json::json!(pptx.slides().len());
            }
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        OutputFormat::Text => {
            println!("File:              {}", args.file.display());
            println!("Format:            {}", format.display_name());

            match &doc {
                AnyDocument::Pdf(_) => {
                    println!("PDF Version:       {}", meta.pdf_version);
                    println!("Pages:             {}", meta.page_count);
                    println!("Encrypted:         {}", meta.is_encrypted);
                }
                AnyDocument::Docx(_) => {
                    println!("Pages:             {} (estimated)", meta.page_count);
                }
                AnyDocument::Xlsx(xlsx) => {
                    println!("Sheets:            {}", xlsx.sheet_names().len());
                    for (i, name) in xlsx.sheet_names().iter().enumerate() {
                        println!("  Sheet {}:         {}", i + 1, name);
                    }
                }
                AnyDocument::Pptx(pptx) => {
                    println!("Slides:            {}", pptx.slides().len());
                }
                AnyDocument::Html(_) => {
                    println!("Pages:             {}", meta.page_count);
                }
                AnyDocument::Epub(epub) => {
                    println!("Chapters:          {}", epub.chapters().len());
                }
            }

            if let Some(ref title) = meta.title {
                println!("Title:             {}", title);
            }
            if let Some(ref author) = meta.author {
                println!("Author:            {}", author);
            }
            if let Some(ref subject) = meta.subject {
                println!("Subject:           {}", subject);
            }
            if let Some(ref keywords) = meta.keywords {
                println!("Keywords:          {}", keywords);
            }
            if let Some(ref creator) = meta.creator {
                println!("Creator:           {}", creator);
            }
            if let Some(ref producer) = meta.producer {
                println!("Producer:          {}", producer);
            }
            if let Some(ref date) = meta.creation_date {
                println!("Created:           {}", date);
            }
            if let Some(ref date) = meta.modification_date {
                println!("Modified:          {}", date);
            }
        }
    }

    Ok(())
}
