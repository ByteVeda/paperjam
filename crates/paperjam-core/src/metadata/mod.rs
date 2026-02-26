pub mod xmp;

use lopdf::Object;

use crate::document::Document;
use crate::error::{PdfError, Result};

/// PDF document metadata.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub pdf_version: String,
    pub page_count: usize,
    pub is_encrypted: bool,
    pub xmp_metadata: Option<String>,
}

/// Fields to update in the PDF /Info dictionary.
///
/// Each field uses `Option<Option<String>>`:
/// - `None` — leave unchanged
/// - `Some(None)` — remove the field
/// - `Some(Some(value))` — set the field to `value`
#[derive(Debug, Clone, Default)]
pub struct MetadataUpdate {
    pub title: Option<Option<String>>,
    pub author: Option<Option<String>>,
    pub subject: Option<Option<String>>,
    pub keywords: Option<Option<String>>,
    pub creator: Option<Option<String>>,
    pub producer: Option<Option<String>>,
}

/// Update the /Info dictionary of a PDF document, returning a new document.
pub fn set_metadata(doc: &Document, update: &MetadataUpdate) -> Result<Document> {
    let mut new_doc = doc.inner().clone();

    // Get or create the /Info dictionary
    let info_id = if let Ok(info_ref) = new_doc.trailer.get(b"Info") {
        info_ref
            .as_reference()
            .map_err(|e| PdfError::Lopdf(e))?
    } else {
        let id = new_doc.add_object(Object::Dictionary(lopdf::Dictionary::new()));
        new_doc.trailer.set("Info", Object::Reference(id));
        id
    };

    let info_obj = new_doc
        .get_object_mut(info_id)
        .map_err(|e| PdfError::Lopdf(e))?;
    let info_dict = info_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Lopdf(e))?;

    apply_field(info_dict, b"Title", &update.title);
    apply_field(info_dict, b"Author", &update.author);
    apply_field(info_dict, b"Subject", &update.subject);
    apply_field(info_dict, b"Keywords", &update.keywords);
    apply_field(info_dict, b"Creator", &update.creator);
    apply_field(info_dict, b"Producer", &update.producer);

    Document::from_lopdf(new_doc)
}

fn apply_field(dict: &mut lopdf::Dictionary, key: &[u8], value: &Option<Option<String>>) {
    match value {
        None => {} // don't touch
        Some(None) => {
            dict.remove(key);
        }
        Some(Some(s)) => {
            let (bytes, format) = encode_pdf_string(s);
            dict.set(key, Object::String(bytes, format));
        }
    }
}

/// Encode a Rust string as a PDF string.
/// Uses UTF-16BE with BOM for non-ASCII, literal bytes otherwise.
fn encode_pdf_string(s: &str) -> (Vec<u8>, lopdf::StringFormat) {
    if s.is_ascii() {
        (s.as_bytes().to_vec(), lopdf::StringFormat::Literal)
    } else {
        let mut bytes = vec![0xFE, 0xFF]; // UTF-16BE BOM
        for code_unit in s.encode_utf16() {
            bytes.push((code_unit >> 8) as u8);
            bytes.push((code_unit & 0xFF) as u8);
        }
        (bytes, lopdf::StringFormat::Hexadecimal)
    }
}

impl Metadata {
    pub fn extract(doc: &lopdf::Document) -> Result<Self> {
        let mut meta = Metadata {
            pdf_version: doc.version.clone(),
            page_count: doc.get_pages().len(),
            is_encrypted: doc.is_encrypted(),
            ..Default::default()
        };

        // Read /Info dictionary from trailer
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok((_, info_obj)) = doc.dereference(info_ref) {
                if let Ok(info_dict) = info_obj.as_dict() {
                    meta.title = get_string_from_dict(doc, info_dict, b"Title");
                    meta.author = get_string_from_dict(doc, info_dict, b"Author");
                    meta.subject = get_string_from_dict(doc, info_dict, b"Subject");
                    meta.keywords = get_string_from_dict(doc, info_dict, b"Keywords");
                    meta.creator = get_string_from_dict(doc, info_dict, b"Creator");
                    meta.producer = get_string_from_dict(doc, info_dict, b"Producer");
                    meta.creation_date = get_string_from_dict(doc, info_dict, b"CreationDate");
                    meta.modification_date = get_string_from_dict(doc, info_dict, b"ModDate");
                }
            }
        }

        meta.xmp_metadata = xmp::extract_xmp(doc);
        Ok(meta)
    }
}

fn get_string_from_dict(
    doc: &lopdf::Document,
    dict: &lopdf::Dictionary,
    key: &[u8],
) -> Option<String> {
    let val = dict.get(key).ok()?;
    let (_, val) = doc.dereference(val).unwrap_or((None, val));

    match val {
        lopdf::Object::String(bytes, _) => Some(decode_pdf_string(bytes)),
        _ => None,
    }
}

fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let code_units: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    ((chunk[0] as u16) << 8) | (chunk[1] as u16)
                } else {
                    chunk[0] as u16
                }
            })
            .collect();
        return String::from_utf16_lossy(&code_units);
    }

    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return String::from_utf8_lossy(&bytes[3..]).to_string();
    }

    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    decoded.to_string()
}
