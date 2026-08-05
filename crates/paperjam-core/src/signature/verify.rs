use lopdf::Object;

use crate::error::{PdfError, Result};
use crate::signature::extract::{extract_byte_range, extract_signed_bytes};
use crate::signature::pkcs7::verify_pkcs7_integrity;
use crate::signature::types::SignatureValidity;

/// Verify all signatures in a PDF document.
///
/// For each signature, checks:
/// 1. Integrity: the hash of the signed bytes matches the PKCS#7 signature
/// 2. Certificate validity: basic date check
pub fn verify_signatures(
    doc: &lopdf::Document,
    raw_bytes: &[u8],
) -> Result<Vec<SignatureValidity>> {
    let root_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Signature("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Signature("/Root not ref".to_string()))?;

    let root_dict = doc
        .get_object(root_id)
        .map_err(|e| PdfError::Signature(format!("Cannot get root: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Signature("/Root not dict".to_string()))?;

    let acroform_dict = match root_dict.get(b"AcroForm") {
        Ok(Object::Dictionary(d)) => d.clone(),
        Ok(Object::Reference(id)) => doc
            .get_object(*id)
            .map_err(|e| PdfError::Signature(format!("AcroForm deref: {}", e)))?
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

    let mut results = Vec::new();
    verify_sig_fields(doc, &fields, &mut results, raw_bytes)?;
    Ok(results)
}

fn verify_sig_fields(
    doc: &lopdf::Document,
    field_refs: &[Object],
    results: &mut Vec<SignatureValidity>,
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

        // Recurse into kids
        if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
            let kids_clone = kids.clone();
            verify_sig_fields(doc, &kids_clone, results, raw_bytes)?;
            continue;
        }

        // Check for signature field
        let is_sig = match dict.get(b"FT") {
            Ok(Object::Name(name)) => name == b"Sig",
            _ => false,
        };
        if !is_sig {
            continue;
        }

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
            .and_then(|o| match o {
                Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "unnamed".to_string());

        let signer = sig_dict.get(b"Name").ok().and_then(|o| match o {
            Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
            _ => None,
        });

        // Get PKCS#7 signature bytes (trim trailing zeros from hex padding)
        let pkcs7_bytes = match sig_dict.get(b"Contents") {
            Ok(Object::String(bytes, _)) => {
                let trimmed = bytes
                    .iter()
                    .rposition(|&b| b != 0)
                    .map(|pos| bytes[..=pos].to_vec())
                    .unwrap_or_default();
                trimmed
            }
            _ => {
                results.push(SignatureValidity {
                    name,
                    integrity_ok: false,
                    certificate_valid: false,
                    message: "No /Contents in signature".to_string(),
                    signer,
                    timestamp_valid: None,
                    revocation_ok: None,
                    is_ltv: false,
                });
                continue;
            }
        };

        // Get ByteRange
        let byte_range = match extract_byte_range(&sig_dict) {
            Some(br) => br,
            None => {
                results.push(SignatureValidity {
                    name,
                    integrity_ok: false,
                    certificate_valid: false,
                    message: "No valid ByteRange".to_string(),
                    signer,
                    timestamp_valid: None,
                    revocation_ok: None,
                    is_ltv: false,
                });
                continue;
            }
        };

        // Extract signed bytes
        let signed_bytes = match extract_signed_bytes(raw_bytes, &byte_range) {
            Ok(bytes) => bytes,
            Err(e) => {
                results.push(SignatureValidity {
                    name,
                    integrity_ok: false,
                    certificate_valid: false,
                    message: format!("Failed to extract signed bytes: {}", e),
                    signer,
                    timestamp_valid: None,
                    revocation_ok: None,
                    is_ltv: false,
                });
                continue;
            }
        };

        // Verify PKCS#7 integrity
        let integrity_ok = match verify_pkcs7_integrity(&pkcs7_bytes, &signed_bytes) {
            Ok(ok) => ok,
            Err(e) => {
                results.push(SignatureValidity {
                    name,
                    integrity_ok: false,
                    certificate_valid: false,
                    message: format!("Integrity check failed: {}", e),
                    signer,
                    timestamp_valid: None,
                    revocation_ok: None,
                    is_ltv: false,
                });
                continue;
            }
        };

        // Check certificate validity (basic date check)
        let cert_valid = check_certificate_dates(&pkcs7_bytes).unwrap_or(false);

        // Check for LTV information
        let has_ts = crate::signature::tsa::has_timestamp_token(&pkcs7_bytes);
        let (has_crls_info, _has_ocsp) = crate::signature::tsa::has_revocation_info(&pkcs7_bytes);
        let is_ltv = has_ts && (has_crls_info || _has_ocsp);

        let message = if integrity_ok && cert_valid {
            if is_ltv {
                "Signature is valid (LTV enabled)".to_string()
            } else {
                "Signature is valid".to_string()
            }
        } else if integrity_ok && !cert_valid {
            "Signature integrity OK but certificate expired or not yet valid".to_string()
        } else {
            "Signature integrity check failed".to_string()
        };

        results.push(SignatureValidity {
            name,
            integrity_ok,
            certificate_valid: cert_valid,
            message,
            signer,
            timestamp_valid: if has_ts { Some(true) } else { None },
            revocation_ok: if has_crls_info { Some(true) } else { None },
            is_ltv,
        });
    }

    Ok(())
}

fn check_certificate_dates(pkcs7_bytes: &[u8]) -> Result<bool> {
    use cms::content_info::ContentInfo;
    use cms::signed_data::SignedData;
    use der::{Decode, Encode};

    let content_info = ContentInfo::from_der(pkcs7_bytes)
        .map_err(|e| PdfError::Signature(format!("CMS parse: {}", e)))?;

    let sd = content_info
        .content
        .decode_as::<SignedData>()
        .map_err(|e| PdfError::Signature(format!("SignedData parse: {}", e)))?;

    let cert_set = sd
        .certificates
        .ok_or_else(|| PdfError::Signature("No certificates".to_string()))?;

    if let Some(cert) = cert_set.0.iter().next() {
        let cert_bytes = cert
            .to_der()
            .map_err(|e| PdfError::Signature(format!("Cert encode: {}", e)))?;
        return crate::signature::cert::is_certificate_date_valid(&cert_bytes);
    }

    Ok(false)
}
