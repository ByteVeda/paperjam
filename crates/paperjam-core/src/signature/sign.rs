use crate::document::Document;
use crate::error::{PdfError, Result};
use crate::signature::pkcs7::create_pkcs7_signature;
use crate::signature::types::SignOptions;

use lopdf::{dictionary, Object};
use sha2::Digest;

/// Sign a PDF document, returning the finalized signed PDF bytes.
///
/// Process:
/// 1. Add a signature field with a placeholder /Contents
/// 2. Serialize the PDF with the placeholder
/// 3. Compute hash over the ByteRange segments
/// 4. Create PKCS#7 signature
/// 5. Embed the real signature in the placeholder
pub fn sign_document(
    doc: &Document,
    private_key_der: &[u8],
    cert_der_list: &[Vec<u8>],
    options: &SignOptions,
) -> Result<Vec<u8>> {
    if cert_der_list.is_empty() {
        return Err(PdfError::Signature(
            "At least one certificate is required".to_string(),
        ));
    }

    // Clone the document for modification
    let mut inner = doc.inner().clone();

    // Larger placeholder for LTV signatures (timestamp tokens, OCSP, certs add size)
    let placeholder_size = if options.ltv.is_some() {
        32768usize
    } else {
        8192usize
    };
    let placeholder = vec![0u8; placeholder_size];

    // Create the signature dictionary
    let sig_dict = dictionary! {
        "Type" => "Sig",
        "Filter" => "Adobe.PPKLite",
        "SubFilter" => "adbe.pkcs7.detached",
        "Contents" => Object::String(placeholder, lopdf::StringFormat::Hexadecimal),
        "ByteRange" => Object::Array(vec![
            Object::Integer(0),
            Object::Integer(9999999999),
            Object::Integer(9999999999),
            Object::Integer(9999999999),
        ]),
    };

    let mut sig_dict_obj = sig_dict;

    if let Some(ref reason) = options.reason {
        sig_dict_obj.set(
            "Reason",
            Object::String(reason.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }
    if let Some(ref location) = options.location {
        sig_dict_obj.set(
            "Location",
            Object::String(location.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }
    if let Some(ref contact) = options.contact_info {
        sig_dict_obj.set(
            "ContactInfo",
            Object::String(contact.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
    }

    // Add signature as object
    let sig_obj_id = inner.new_object_id();
    inner
        .objects
        .insert(sig_obj_id, Object::Dictionary(sig_dict_obj));

    // Create signature field
    let sig_field = dictionary! {
        "Type" => "Annot",
        "Subtype" => "Widget",
        "FT" => "Sig",
        "T" => Object::String(
            options.field_name.as_bytes().to_vec(),
            lopdf::StringFormat::Literal,
        ),
        "V" => Object::Reference(sig_obj_id),
        "F" => Object::Integer(132), // Hidden + Print
        "Rect" => Object::Array(vec![
            Object::Integer(0), Object::Integer(0),
            Object::Integer(0), Object::Integer(0),
        ]),
    };

    let field_id = inner.new_object_id();
    inner
        .objects
        .insert(field_id, Object::Dictionary(sig_field));

    // Add field to AcroForm
    ensure_acroform_field(&mut inner, field_id)?;

    // Add annotation to first page
    add_annot_to_first_page(&mut inner, field_id)?;

    // Serialize with placeholder
    let mut pdf_bytes = Vec::new();
    inner
        .save_to(&mut pdf_bytes)
        .map_err(|e| PdfError::Signature(format!("Failed to serialize PDF: {}", e)))?;

    // Find the /Contents placeholder in the serialized bytes
    // lopdf writes hex strings without a separator: /Contents<hex>
    let contents_marker = b"/Contents<";
    let contents_pos = find_subsequence(&pdf_bytes, contents_marker)
        .ok_or_else(|| PdfError::Signature("Cannot find /Contents placeholder".to_string()))?;

    let hex_start = contents_pos + contents_marker.len();
    // Find the closing >
    let hex_end = pdf_bytes[hex_start..]
        .iter()
        .position(|&b| b == b'>')
        .map(|p| hex_start + p)
        .ok_or_else(|| PdfError::Signature("Cannot find end of /Contents hex".to_string()))?;

    // Compute byte range
    let byte_range = [
        0i64,
        hex_start as i64 - 1, // up to and including '<'
        (hex_end + 1) as i64, // after '>'
        (pdf_bytes.len() as i64) - (hex_end + 1) as i64,
    ];

    // Update ByteRange in the PDF
    // lopdf writes arrays without a separator: /ByteRange[...]
    let byterange_marker = b"/ByteRange[";
    if let Some(br_pos) = find_subsequence(&pdf_bytes, byterange_marker) {
        let br_start = br_pos + byterange_marker.len();
        let br_end = pdf_bytes[br_start..]
            .iter()
            .position(|&b| b == b']')
            .map(|p| br_start + p)
            .ok_or_else(|| PdfError::Signature("Cannot find ByteRange end".to_string()))?;

        let br_str = format!(
            "{} {} {} {}",
            byte_range[0], byte_range[1], byte_range[2], byte_range[3]
        );
        let br_bytes = br_str.as_bytes();

        // Pad with spaces to fill the same length
        let available = br_end - br_start;
        let mut padded = vec![b' '; available];
        padded[..br_bytes.len().min(available)]
            .copy_from_slice(&br_bytes[..br_bytes.len().min(available)]);
        pdf_bytes[br_start..br_end].copy_from_slice(&padded);
    }

    // Extract signed bytes
    let mut signed_data = Vec::new();
    signed_data.extend_from_slice(&pdf_bytes[..hex_start - 1]);
    signed_data.extend_from_slice(&pdf_bytes[hex_end + 1..]);

    // Hash the signed data
    let hash = sha2::Sha256::digest(&signed_data);

    // Create PKCS#7 signature with full cert chain and signed attributes
    let mut pkcs7 = create_pkcs7_signature(&hash, private_key_der, cert_der_list, None)?;

    // Handle LTV: add timestamp token if requested
    if let Some(ref ltv) = options.ltv {
        // Get the timestamp token
        let ts_token = if let Some(ref pre_fetched) = ltv.timestamp_token {
            Some(pre_fetched.clone())
        } else if let Some(ref tsa_url) = ltv.tsa_url {
            // Extract the signature value from the CMS for timestamping
            #[cfg(feature = "ltv")]
            {
                Some(crate::signature::tsa::fetch_timestamp_token(
                    tsa_url, &pkcs7,
                )?)
            }
            #[cfg(not(feature = "ltv"))]
            {
                let _ = tsa_url;
                return Err(PdfError::Signature(
                    "LTV feature not enabled; cannot fetch timestamp from TSA".to_string(),
                ));
            }
        } else {
            None
        };

        if let Some(token) = ts_token {
            pkcs7 = crate::signature::pkcs7::add_timestamp_to_cms(&pkcs7, &token)?;
        }
    }

    if pkcs7.len() > placeholder_size {
        return Err(PdfError::Signature(format!(
            "Signature too large ({} bytes, max {})",
            pkcs7.len(),
            placeholder_size
        )));
    }

    // Hex-encode the signature
    let hex_sig = hex_encode(&pkcs7);
    // Pad with zeros to fill the placeholder
    let padded_hex = format!("{:0<width$}", hex_sig, width = (hex_end - hex_start));

    // Write the signature into the PDF
    pdf_bytes[hex_start..hex_end].copy_from_slice(padded_hex.as_bytes());

    Ok(pdf_bytes)
}

fn ensure_acroform_field(doc: &mut lopdf::Document, field_id: lopdf::ObjectId) -> Result<()> {
    let root_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| PdfError::Signature("No /Root".to_string()))?
        .as_reference()
        .map_err(|_| PdfError::Signature("/Root not ref".to_string()))?;

    let root_dict = doc
        .get_object(root_id)
        .map_err(|e| PdfError::Signature(format!("Root get: {}", e)))?
        .as_dict()
        .map_err(|_| PdfError::Signature("/Root not dict".to_string()))?;

    // Check if AcroForm exists
    let acroform_ref = root_dict.get(b"AcroForm").ok().and_then(|o| match o {
        Object::Reference(id) => Some(*id),
        _ => None,
    });

    if let Some(af_id) = acroform_ref {
        let af_obj = doc
            .get_object_mut(af_id)
            .map_err(|e| PdfError::Signature(format!("AcroForm get: {}", e)))?;
        let af_dict = af_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Signature("AcroForm not dict".to_string()))?;

        match af_dict.get_mut(b"Fields") {
            Ok(Object::Array(fields)) => {
                fields.push(Object::Reference(field_id));
            }
            _ => {
                af_dict.set("Fields", Object::Array(vec![Object::Reference(field_id)]));
            }
        }
        // Set SigFlags to indicate signatures exist
        af_dict.set("SigFlags", Object::Integer(3));
    } else {
        // Create AcroForm
        let acroform = dictionary! {
            "Fields" => Object::Array(vec![Object::Reference(field_id)]),
            "SigFlags" => Object::Integer(3),
        };
        let af_id = doc.new_object_id();
        doc.objects.insert(af_id, Object::Dictionary(acroform));

        let root_obj = doc
            .get_object_mut(root_id)
            .map_err(|e| PdfError::Signature(format!("Root get: {}", e)))?;
        let root_dict = root_obj
            .as_dict_mut()
            .map_err(|_| PdfError::Signature("/Root not dict".to_string()))?;
        root_dict.set("AcroForm", Object::Reference(af_id));
    }

    Ok(())
}

fn add_annot_to_first_page(doc: &mut lopdf::Document, field_id: lopdf::ObjectId) -> Result<()> {
    let pages = doc.get_pages();
    let first_page_id = pages
        .get(&1)
        .ok_or_else(|| PdfError::Signature("No first page".to_string()))?;

    let page_obj = doc
        .get_object_mut(*first_page_id)
        .map_err(|e| PdfError::Signature(format!("Page get: {}", e)))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|_| PdfError::Signature("Page not dict".to_string()))?;

    match page_dict.get_mut(b"Annots") {
        Ok(Object::Array(annots)) => {
            annots.push(Object::Reference(field_id));
        }
        _ => {
            page_dict.set("Annots", Object::Array(vec![Object::Reference(field_id)]));
        }
    }

    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}
