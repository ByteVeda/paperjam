use paperjam_model::metadata::Metadata;
use std::io::{Cursor, Read};

use crate::document::DocxDocument;
use crate::error::DocxError;

impl DocxDocument {
    /// Extract document metadata.
    ///
    /// `docx-rs` does not expose the core properties (title, author, etc.)
    /// that are stored in `docProps/core.xml` inside the ZIP. We open the
    /// raw bytes as a ZIP archive and manually parse the core properties
    /// XML with simple string extraction.
    pub fn extract_metadata(&self) -> Result<Metadata, DocxError> {
        let core_xml = read_core_xml_from_zip(&self.raw_bytes);
        let mut meta = Metadata {
            pdf_version: String::new(),
            is_encrypted: false,
            page_count: 1, // DOCX doesn't have an exact page count without rendering
            ..Default::default()
        };

        if let Some(xml) = core_xml {
            meta.title = extract_element(&xml, "dc:title");
            meta.author = extract_element(&xml, "dc:creator");
            meta.subject = extract_element(&xml, "dc:subject");
            meta.keywords = extract_element(&xml, "cp:keywords");
            meta.creator = extract_element(&xml, "dc:creator");
            meta.creation_date = extract_element(&xml, "dcterms:created");
            meta.modification_date = extract_element(&xml, "dcterms:modified");
        }

        Ok(meta)
    }
}

/// Read `docProps/core.xml` from the DOCX ZIP archive.
fn read_core_xml_from_zip(bytes: &[u8]) -> Option<String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name("docProps/core.xml").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    Some(contents)
}

/// Extract the text content of an XML element with the given tag name.
///
/// This is a simple approach that avoids pulling in a full XML parser.
/// It handles the common case `<tag ...>text</tag>`.
fn extract_element(xml: &str, tag: &str) -> Option<String> {
    // Look for opening tag (may have attributes)
    let open_start = xml.find(&format!("<{}", tag))?;
    let after_open = &xml[open_start..];
    let content_start = after_open.find('>')? + 1;
    let content_str = &after_open[content_start..];

    let close_tag = format!("</{}>", tag);
    let end = content_str.find(&close_tag)?;
    let value = content_str[..end].trim().to_string();

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
