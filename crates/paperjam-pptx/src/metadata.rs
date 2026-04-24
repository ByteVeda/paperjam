use crate::error::{PptxError, Result};
use paperjam_model::zip_safety::{SafeArchive, ZipSafetyError};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Read, Seek};

/// Internal representation of PPTX metadata extracted from `docProps/core.xml`.
#[derive(Debug, Clone, Default)]
pub struct PptxMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
}

impl PptxMetadata {
    /// Convert to the shared paperjam-model `Metadata` type.
    pub fn to_model_metadata(&self, slide_count: usize) -> paperjam_model::metadata::Metadata {
        paperjam_model::metadata::Metadata {
            title: self.title.clone(),
            author: self.author.clone(),
            subject: self.subject.clone(),
            keywords: None,
            creator: None,
            producer: Some("paperjam-pptx".to_string()),
            creation_date: self.creation_date.clone(),
            modification_date: self.modification_date.clone(),
            pdf_version: String::new(),
            page_count: slide_count,
            is_encrypted: false,
            xmp_metadata: None,
        }
    }
}

/// Parse metadata from `docProps/core.xml` inside the PPTX ZIP archive.
pub fn parse_metadata<R: Read + Seek>(safe: &mut SafeArchive<'_, R>) -> Result<PptxMetadata> {
    // A DOCX/PPTX produced without core.xml is legal (the file is just
    // missing metadata); only the `MissingEntry` flavour of the safety
    // error counts as a graceful fallback here.
    let xml = match safe.read_entry_string("docProps/core.xml") {
        Ok(buf) => buf,
        Err(ZipSafetyError::MissingEntry(_)) => return Ok(PptxMetadata::default()),
        Err(e) => return Err(e.into()),
    };

    let mut reader = Reader::from_str(&xml);
    let mut meta = PptxMetadata::default();
    let mut current_tag: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "title" | "creator" | "subject" | "created" | "modified" => {
                        current_tag = Some(local);
                    }
                    _ => {
                        current_tag = None;
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref tag) = current_tag {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if !text.is_empty() {
                        match tag.as_str() {
                            "title" => meta.title = Some(text),
                            "creator" => meta.author = Some(text),
                            "subject" => meta.subject = Some(text),
                            "created" => meta.creation_date = Some(text),
                            "modified" => meta.modification_date = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PptxError::Xml(format!("metadata XML error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(meta)
}

/// Extract the local part of a possibly namespaced XML tag name.
///
/// e.g., `b"dc:title"` -> `"title"`, `b"cp:coreProperties"` -> `"coreProperties"`.
fn local_name(full: &[u8]) -> String {
    let s = std::str::from_utf8(full).unwrap_or("");
    match s.rfind(':') {
        Some(pos) => s[pos + 1..].to_string(),
        None => s.to_string(),
    }
}
