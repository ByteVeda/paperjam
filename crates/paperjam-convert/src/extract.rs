use paperjam_model::format::DocumentFormat;

use crate::error::ConvertError;
use crate::intermediate::IntermediateDoc;

/// Extract an [`IntermediateDoc`] from bytes in the given format.
#[allow(unused_variables)]
pub fn extract(bytes: &[u8], format: DocumentFormat) -> Result<IntermediateDoc, ConvertError> {
    match format {
        #[cfg(feature = "pdf")]
        DocumentFormat::Pdf => extract_pdf(bytes),

        #[cfg(feature = "docx")]
        DocumentFormat::Docx => extract_docx(bytes),

        #[cfg(feature = "xlsx")]
        DocumentFormat::Xlsx => extract_xlsx(bytes),

        #[cfg(feature = "pptx")]
        DocumentFormat::Pptx => extract_pptx(bytes),

        #[cfg(feature = "html")]
        DocumentFormat::Html => extract_html(bytes),

        #[cfg(feature = "epub")]
        DocumentFormat::Epub => extract_epub(bytes),

        _ => Err(ConvertError::unsupported(format)),
    }
}

// ---------------------------------------------------------------------------
// PDF
// ---------------------------------------------------------------------------

#[cfg(feature = "pdf")]
fn extract_pdf(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_core::document::Document;
    use paperjam_core::structure::{extract_document_structure, StructureOptions};
    use paperjam_core::table::TableExtractionOptions;

    let doc = Document::open_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().map(|m| (*m).clone()).unwrap_or_default();

    let options = StructureOptions::default();
    let blocks = extract_document_structure(&doc, &options)
        .map_err(|e| ConvertError::Extraction(e.to_string()))?;

    // Collect tables from all pages.
    let table_opts = TableExtractionOptions::default();
    let mut tables = Vec::new();
    for page_num in 1..=doc.page_count() as u32 {
        if let Ok(page) = doc.page(page_num) {
            if let Ok(page_tables) = page.extract_tables(&table_opts) {
                tables.extend(page_tables);
            }
        }
    }

    // Collect images from all pages.
    let mut images = Vec::new();
    for page_num in 1..=doc.page_count() as u32 {
        if let Ok(page_images) = doc.extract_images(page_num) {
            images.extend(page_images);
        }
    }

    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}

// ---------------------------------------------------------------------------
// DOCX
// ---------------------------------------------------------------------------

#[cfg(feature = "docx")]
fn extract_docx(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_docx::DocxDocument;
    use paperjam_model::document::DocumentTrait;

    let doc =
        DocxDocument::from_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().unwrap_or_default();

    let blocks = doc
        .extract_structure()
        .map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let tables = doc.extract_tables().unwrap_or_default();

    let images = doc.extract_images().unwrap_or_default();

    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}

// ---------------------------------------------------------------------------
// XLSX
// ---------------------------------------------------------------------------

#[cfg(feature = "xlsx")]
fn extract_xlsx(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_model::document::DocumentTrait;
    use paperjam_xlsx::XlsxDocument;

    let doc =
        XlsxDocument::open_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().unwrap_or_default();

    let blocks = doc.extract_structure().unwrap_or_default();

    let tables = doc.extract_tables().unwrap_or_default();

    let images = doc.extract_images().unwrap_or_default();

    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}

// ---------------------------------------------------------------------------
// PPTX
// ---------------------------------------------------------------------------

#[cfg(feature = "pptx")]
fn extract_pptx(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_model::document::DocumentTrait;
    use paperjam_pptx::PptxDocument;

    let doc =
        PptxDocument::from_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().unwrap_or_default();

    let blocks = doc.extract_structure().unwrap_or_default();

    let tables = doc.extract_tables().unwrap_or_default();

    let images = doc.extract_images().unwrap_or_default();

    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}

// ---------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------

#[cfg(feature = "html")]
fn extract_html(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_html::HtmlDocument;
    use paperjam_model::document::DocumentTrait;

    let doc =
        HtmlDocument::from_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().unwrap_or_default();
    let blocks = doc
        .extract_structure()
        .map_err(|e| ConvertError::Extraction(e.to_string()))?;
    let tables = doc.extract_tables().unwrap_or_default();
    let images = doc.extract_images().unwrap_or_default();
    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}

// ---------------------------------------------------------------------------
// EPUB
// ---------------------------------------------------------------------------

#[cfg(feature = "epub")]
fn extract_epub(bytes: &[u8]) -> Result<IntermediateDoc, ConvertError> {
    use paperjam_epub::EpubDocument;
    use paperjam_model::document::DocumentTrait;

    let doc =
        EpubDocument::from_bytes(bytes).map_err(|e| ConvertError::Extraction(e.to_string()))?;

    let metadata = doc.metadata().unwrap_or_default();
    let blocks = doc.extract_structure().unwrap_or_default();
    let tables = doc.extract_tables().unwrap_or_default();
    let images = doc.extract_images().unwrap_or_default();
    let bookmarks = doc.bookmarks().unwrap_or_default();

    Ok(IntermediateDoc {
        metadata,
        blocks,
        tables,
        images,
        bookmarks,
    })
}
