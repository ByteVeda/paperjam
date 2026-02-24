pub mod xmp;

use crate::error::Result;

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
