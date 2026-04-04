use std::path::Path;

use paperjam_core::document::Document;
use paperjam_model::format::DocumentFormat;
use paperjam_model::metadata::Metadata;
use paperjam_model::structure::ContentBlock;
use paperjam_model::table::Table;

use crate::error::CliError;

/// Open a PDF document (used by PDF-specific commands).
pub fn open_document(path: &Path, password: Option<&str>) -> Result<Document, CliError> {
    if !path.exists() {
        return Err(CliError::FileNotFound(path.to_path_buf()));
    }
    let doc = match password {
        Some(pw) => Document::open_with_password(path, pw),
        None => Document::open(path),
    };
    doc.map_err(CliError::Pdf)
}

/// A document in any supported format.
#[allow(clippy::large_enum_variant)]
pub enum AnyDocument {
    Pdf(Document),
    Docx(paperjam_docx::DocxDocument),
    Xlsx(paperjam_xlsx::XlsxDocument),
    Pptx(paperjam_pptx::PptxDocument),
    Html(paperjam_html::HtmlDocument),
    Epub(paperjam_epub::EpubDocument),
}

impl AnyDocument {
    /// Return the document format.
    pub fn format(&self) -> DocumentFormat {
        match self {
            AnyDocument::Pdf(_) => DocumentFormat::Pdf,
            AnyDocument::Docx(_) => DocumentFormat::Docx,
            AnyDocument::Xlsx(_) => DocumentFormat::Xlsx,
            AnyDocument::Pptx(_) => DocumentFormat::Pptx,
            AnyDocument::Html(_) => DocumentFormat::Html,
            AnyDocument::Epub(_) => DocumentFormat::Epub,
        }
    }

    /// Extract all text as a single string.
    pub fn extract_text(&self) -> Result<String, CliError> {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => {
                let total = doc.page_count() as u32;
                let mut texts = Vec::new();
                for page_num in 1..=total {
                    let page = doc.page(page_num)?;
                    texts.push(page.extract_text()?);
                }
                Ok(texts.join("\n\n"))
            }
            AnyDocument::Docx(doc) => doc.extract_text().map_err(CliError::Docx),
            AnyDocument::Xlsx(doc) => doc.extract_text().map_err(CliError::Xlsx),
            AnyDocument::Pptx(doc) => doc.extract_text().map_err(CliError::Pptx),
            AnyDocument::Html(doc) => doc.extract_text().map_err(CliError::Html),
            AnyDocument::Epub(doc) => doc.extract_text().map_err(CliError::Epub),
        }
    }

    /// Extract all tables.
    pub fn extract_tables(&self) -> Result<Vec<Table>, CliError> {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => {
                let total = doc.page_count() as u32;
                let opts = paperjam_core::table::TableExtractionOptions::default();
                let mut all_tables = Vec::new();
                for page_num in 1..=total {
                    let page = doc.page(page_num)?;
                    all_tables.extend(page.extract_tables(&opts)?);
                }
                Ok(all_tables)
            }
            AnyDocument::Docx(doc) => doc.extract_tables().map_err(CliError::Docx),
            AnyDocument::Xlsx(doc) => doc.extract_tables().map_err(CliError::Xlsx),
            AnyDocument::Pptx(doc) => doc.extract_tables().map_err(CliError::Pptx),
            AnyDocument::Html(doc) => doc.extract_tables().map_err(CliError::Html),
            AnyDocument::Epub(doc) => doc.extract_tables().map_err(CliError::Epub),
        }
    }

    /// Extract document structure.
    pub fn extract_structure(&self) -> Result<Vec<ContentBlock>, CliError> {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => {
                let opts = paperjam_core::structure::StructureOptions::default();
                let blocks = paperjam_core::structure::extract_document_structure(doc, &opts)?;
                Ok(blocks)
            }
            AnyDocument::Docx(doc) => doc.extract_structure().map_err(CliError::Docx),
            AnyDocument::Xlsx(doc) => doc.extract_structure().map_err(CliError::Xlsx),
            AnyDocument::Pptx(doc) => doc.extract_structure().map_err(CliError::Pptx),
            AnyDocument::Html(doc) => doc.extract_structure().map_err(CliError::Html),
            AnyDocument::Epub(doc) => doc.extract_structure().map_err(CliError::Epub),
        }
    }

    /// Extract metadata.
    pub fn metadata(&self) -> Result<Metadata, CliError> {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => {
                let arc = doc.metadata().map_err(CliError::Pdf)?;
                Ok((*arc).clone())
            }
            AnyDocument::Docx(doc) => doc.metadata().map_err(CliError::Docx),
            AnyDocument::Xlsx(doc) => doc.metadata().map_err(CliError::Xlsx),
            AnyDocument::Pptx(doc) => doc.metadata().map_err(CliError::Pptx),
            AnyDocument::Html(doc) => doc.metadata().map_err(CliError::Html),
            AnyDocument::Epub(doc) => doc.metadata().map_err(CliError::Epub),
        }
    }

    /// Convert to markdown.
    pub fn to_markdown(&self) -> Result<String, CliError> {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => {
                let opts = paperjam_core::markdown::MarkdownOptions::default();
                paperjam_core::markdown::document_to_markdown(doc, &opts).map_err(CliError::Pdf)
            }
            AnyDocument::Docx(doc) => doc.to_markdown().map_err(CliError::Docx),
            AnyDocument::Xlsx(doc) => doc.to_markdown().map_err(CliError::Xlsx),
            AnyDocument::Pptx(doc) => doc.to_markdown().map_err(CliError::Pptx),
            AnyDocument::Html(doc) => doc.to_markdown().map_err(CliError::Html),
            AnyDocument::Epub(doc) => doc.to_markdown().map_err(CliError::Epub),
        }
    }

    /// Return page count (or sheet/slide count).
    #[allow(dead_code)]
    pub fn page_count(&self) -> usize {
        use paperjam_model::document::DocumentTrait;
        match self {
            AnyDocument::Pdf(doc) => doc.page_count(),
            AnyDocument::Docx(doc) => doc.page_count(),
            AnyDocument::Xlsx(doc) => doc.page_count(),
            AnyDocument::Pptx(doc) => doc.page_count(),
            AnyDocument::Html(doc) => doc.page_count(),
            AnyDocument::Epub(doc) => doc.page_count(),
        }
    }
}

/// Open any supported document format, detecting the format from the file extension.
pub fn open_any(path: &Path, password: Option<&str>) -> Result<AnyDocument, CliError> {
    if !path.exists() {
        return Err(CliError::FileNotFound(path.to_path_buf()));
    }

    let format = DocumentFormat::detect(path);

    match format {
        DocumentFormat::Pdf => {
            let doc = open_document(path, password)?;
            Ok(AnyDocument::Pdf(doc))
        }
        DocumentFormat::Docx => {
            let bytes = std::fs::read(path)?;
            let doc = paperjam_docx::DocxDocument::from_bytes(&bytes)?;
            Ok(AnyDocument::Docx(doc))
        }
        DocumentFormat::Xlsx => {
            let doc = paperjam_xlsx::XlsxDocument::open(path)?;
            Ok(AnyDocument::Xlsx(doc))
        }
        DocumentFormat::Pptx => {
            let bytes = std::fs::read(path)?;
            let doc = paperjam_pptx::PptxDocument::from_bytes(&bytes)?;
            Ok(AnyDocument::Pptx(doc))
        }
        DocumentFormat::Html => {
            let bytes = std::fs::read(path)?;
            let doc = paperjam_html::HtmlDocument::from_bytes(&bytes)?;
            Ok(AnyDocument::Html(doc))
        }
        DocumentFormat::Epub => {
            let bytes = std::fs::read(path)?;
            let doc = paperjam_epub::EpubDocument::from_bytes(&bytes)?;
            Ok(AnyDocument::Epub(doc))
        }
        _ => Err(CliError::InvalidArgument(format!(
            "Unsupported file format: {}",
            path.display()
        ))),
    }
}
