use lopdf::Object;

use crate::error::{PdfError, Result};
use crate::signature::cert::parse_certificate_info;
use crate::signature::types::SignatureInfo;

/// Extract all digital signatures from a PDF document's bytes.
///
/// We work with the raw bytes because signature verification needs the
/// original byte offsets from the ByteRange.
pub fn extract_signatures(
    doc: &lopdf::Document,
    raw_bytes: &[u8],
) -> Result<Vec<SignatureInfo>> {
    let root_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Signature("No /Root in trailer".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Signature("/Root is not a reference".to_string()))?;

    let root_dict = doc
        .get_object(root_id)
        .map_err(|e| PdfError::Signature(format!("Cannot get root: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Signature("/Root is not a dict".to_string()))?;

    let acroform_dict = match root_dict.get(b"AcroForm") {
        Ok(Object::Dictionary(d)) => d.clone(),
        Ok(Object::Reference(id)) => doc
            .get_object(*id)
            .map_err(|e| PdfError::Signature(format!("Cannot get AcroForm: {}", e)))?
            .as_dict()
            .map_err(|_| PdfError::Signature("AcroForm not dict".to_string()))?
            .clone(),
        _ => return Ok(Vec::new()),
    };

    let fields = match acroform_dict.get(b"Fields") {
        Ok(Object::Array(arr)) => arr.clone(),
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(Object::Array(arr)) => arr.clone(),
            _ => return Ok(Vec::new()),
        },
        _ => return Ok(Vec::new()),
    };

    let mut signatures = Vec::new();
    collect_sig_fields(doc, &fields, &mut signatures, raw_bytes)?;
    Ok(signatures)
}

/// Recursively search the field tree for signature fields (/FT = /Sig with /V).
fn collect_sig_fields(
    doc: &lopdf::Document,
    field_refs: &[Object],
    results: &mut Vec<SignatureInfo>,
    raw_bytes: &[u8],
) -> Result<()> {
    for field_ref in field_refs {
        let field_id = match field_ref.as_reference() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let dict = match doc.get_object(field_id) {
            Ok(obj) => match obj.as_dict() {
                Ok(d) => d.clone(),
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        // Check for /Kids (recurse)
        if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
            let kids_clone = kids.clone();
            collect_sig_fields(doc, &kids_clone, results, raw_bytes)?;
            continue;
        }

        // Check if this is a signature field
        let is_sig = match dict.get(b"FT") {
            Ok(Object::Name(name)) => name == b"Sig",
            _ => false,
        };
        if !is_sig {
            continue;
        }

        // Get the signature value dictionary
        let sig_dict = match dict.get(b"V") {
            Ok(Object::Dictionary(d)) => d.clone(),
            Ok(Object::Reference(id)) => match doc.get_object(*id) {
                Ok(obj) => match obj.as_dict() {
                    Ok(d) => d.clone(),
                    Err(_) => continue,
                },
                Err(_) => continue,
            },
            _ => continue,
        };

        let name = dict
            .get(b"T")
            .ok()
            .and_then(|o| obj_to_string(o, doc))
            .unwrap_or_else(|| "unnamed".to_string());

        let signer = sig_dict
            .get(b"Name")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        let reason = sig_dict
            .get(b"Reason")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        let location = sig_dict
            .get(b"Location")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        let date = sig_dict
            .get(b"M")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        let contact_info = sig_dict
            .get(b"ContactInfo")
            .ok()
            .and_then(|o| obj_to_string(o, doc));

        let byte_range = extract_byte_range(&sig_dict);
        let covers_whole = byte_range
            .map(|br| {
                let end = br[2] + br[3];
                end as usize >= raw_bytes.len()
            })
            .unwrap_or(false);

        // Try to extract certificate from /Contents (PKCS#7 signature)
        // Trim trailing zeros from hex placeholder padding
        let certificate = sig_dict
            .get(b"Contents")
            .ok()
            .and_then(|o| match o {
                Object::String(bytes, _) => {
                    let trimmed: Vec<u8> = bytes.iter().copied()
                        .rev().skip_while(|&b| b == 0).collect::<Vec<_>>()
                        .into_iter().rev().collect();
                    parse_certificate_info(&trimmed).ok()
                }
                _ => None,
            });

        results.push(SignatureInfo {
            name,
            signer,
            reason,
            location,
            date,
            contact_info,
            byte_range,
            certificate,
            covers_whole_document: covers_whole,
        });
    }

    Ok(())
}

/// Extract the ByteRange array from a signature dictionary.
pub fn extract_byte_range(sig_dict: &lopdf::Dictionary) -> Option<[i64; 4]> {
    match sig_dict.get(b"ByteRange") {
        Ok(Object::Array(arr)) if arr.len() == 4 => {
            let mut br = [0i64; 4];
            for (i, v) in arr.iter().enumerate() {
                br[i] = match v {
                    Object::Integer(n) => *n,
                    _ => return None,
                };
            }
            Some(br)
        }
        _ => None,
    }
}

/// Extract the signed bytes from the PDF using the ByteRange.
pub fn extract_signed_bytes(raw_bytes: &[u8], byte_range: &[i64; 4]) -> Result<Vec<u8>> {
    let mut signed = Vec::new();

    let offset1 = byte_range[0] as usize;
    let len1 = byte_range[1] as usize;
    let offset2 = byte_range[2] as usize;
    let len2 = byte_range[3] as usize;

    if offset1 + len1 > raw_bytes.len() || offset2 + len2 > raw_bytes.len() {
        return Err(PdfError::Signature("ByteRange extends beyond file".to_string()));
    }

    signed.extend_from_slice(&raw_bytes[offset1..offset1 + len1]);
    signed.extend_from_slice(&raw_bytes[offset2..offset2 + len2]);

    Ok(signed)
}

fn obj_to_string(obj: &Object, doc: &lopdf::Document) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).to_string()),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| obj_to_string(o, doc)),
        _ => None,
    }
}
