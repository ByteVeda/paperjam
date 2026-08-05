//! RFC 3161 Timestamp Authority (TSA) client.
//!
//! Builds timestamp requests, parses responses, and optionally fetches tokens
//! from a TSA server (behind the `ltv` feature flag).

use crate::error::{PdfError, Result};
use der::{Decode, Encode, Sequence};
use sha2::Digest;
use spki::AlgorithmIdentifierOwned;

/// OID for id-smime-aa-timeStampToken: 1.2.840.113549.1.9.16.2.14
pub const TIMESTAMP_TOKEN_OID: &str = "1.2.840.113549.1.9.16.2.14";

/// MessageImprint as defined in RFC 3161.
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
struct MessageImprint {
    pub hash_algorithm: AlgorithmIdentifierOwned,
    pub hashed_message: der::asn1::OctetString,
}

/// Build an RFC 3161 timestamp request for the given signature value.
pub fn build_timestamp_request(signature_value: &[u8]) -> Result<Vec<u8>> {
    let hash = sha2::Sha256::digest(signature_value);

    let sha256_oid = der::oid::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");

    let msg_imprint = MessageImprint {
        hash_algorithm: AlgorithmIdentifierOwned {
            oid: sha256_oid,
            parameters: None,
        },
        hashed_message: der::asn1::OctetString::new(hash.to_vec())
            .map_err(|e| PdfError::Signature(format!("TSA: OctetString error: {}", e)))?,
    };

    let msg_imprint_der = msg_imprint
        .to_der()
        .map_err(|e| PdfError::Signature(format!("TSA: MessageImprint encode: {}", e)))?;

    // Build TimeStampReq manually:
    // SEQUENCE { version INTEGER(1), messageImprint, certReq BOOLEAN(TRUE) }
    let mut req_content = Vec::new();
    req_content.extend_from_slice(&[0x02, 0x01, 0x01]); // version INTEGER 1
    req_content.extend_from_slice(&msg_imprint_der); // messageImprint
    req_content.extend_from_slice(&[0x01, 0x01, 0xFF]); // certReq BOOLEAN TRUE

    let mut request = Vec::new();
    request.push(0x30); // SEQUENCE tag
    encode_der_length(req_content.len(), &mut request);
    request.extend_from_slice(&req_content);

    Ok(request)
}

/// Parse an RFC 3161 timestamp response and extract the TimeStampToken.
///
/// The response is: `SEQUENCE { status PKIStatusInfo, timeStampToken [OPTIONAL] }`
/// We check the status integer and return the token bytes.
pub fn parse_timestamp_response(resp_bytes: &[u8]) -> Result<Vec<u8>> {
    // Minimal DER parsing: skip the outer SEQUENCE, read PKIStatusInfo, extract token
    let (_, content) =
        parse_der_sequence(resp_bytes).map_err(|e| PdfError::Signature(format!("TSA: {}", e)))?;

    // First element: PKIStatusInfo (SEQUENCE)
    let (status_info_bytes, rest) = parse_der_element(content)
        .map_err(|e| PdfError::Signature(format!("TSA status parse: {}", e)))?;

    // Parse status integer from PKIStatusInfo
    let (_, status_content) = parse_der_sequence(status_info_bytes)
        .map_err(|e| PdfError::Signature(format!("TSA PKIStatusInfo: {}", e)))?;
    let (status_elem, _) = parse_der_element(status_content)
        .map_err(|e| PdfError::Signature(format!("TSA status int: {}", e)))?;

    // Parse the integer value
    if status_elem.is_empty() || status_elem[0] != 0x02 {
        return Err(PdfError::Signature(
            "TSA: expected INTEGER for status".to_string(),
        ));
    }
    let (_, int_bytes) = parse_der_element(status_elem)
        .map_err(|e| PdfError::Signature(format!("TSA status value: {}", e)))?;
    let status = int_bytes.iter().fold(0i64, |acc, &b| (acc << 8) | b as i64);

    if status > 1 {
        return Err(PdfError::Signature(format!(
            "TSA request rejected with status {}",
            status
        )));
    }

    if rest.is_empty() {
        return Err(PdfError::Signature(
            "TSA response contains no timestamp token".to_string(),
        ));
    }

    // The rest is the TimeStampToken (a DER-encoded ContentInfo)
    let (token_bytes, _) = parse_der_element(rest)
        .map_err(|e| PdfError::Signature(format!("TSA token parse: {}", e)))?;

    Ok(token_bytes.to_vec())
}

/// Fetch a timestamp token from a TSA server.
#[cfg(feature = "ltv")]
pub fn fetch_timestamp_token(tsa_url: &str, signature_value: &[u8]) -> Result<Vec<u8>> {
    use std::sync::Arc;
    use ureq::tls::{TlsConfig, TlsProvider};

    let request_bytes = build_timestamp_request(signature_value)?;

    let crypto = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let agent = ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .unversioned_rustls_crypto_provider(crypto)
                .build(),
        )
        .build()
        .new_agent();

    let response = agent
        .post(tsa_url)
        .header("Content-Type", "application/timestamp-query")
        .send(&request_bytes)
        .map_err(|e| PdfError::Signature(format!("TSA HTTP request failed: {}", e)))?;

    let resp_bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| PdfError::Signature(format!("TSA response read error: {}", e)))?;

    parse_timestamp_response(&resp_bytes)
}

/// Check if a CMS SignedData contains a timestamp token as an unsigned attribute.
pub fn has_timestamp_token(pkcs7_bytes: &[u8]) -> bool {
    use cms::content_info::ContentInfo;
    use cms::signed_data::SignedData;

    let ci = match ContentInfo::from_der(pkcs7_bytes) {
        Ok(ci) => ci,
        Err(_) => return false,
    };

    let sd = match ci.content.decode_as::<SignedData>() {
        Ok(sd) => sd,
        Err(_) => return false,
    };

    let signer_info = match sd.signer_infos.0.iter().next() {
        Some(si) => si,
        None => return false,
    };

    let ts_oid = der::oid::ObjectIdentifier::new_unwrap(TIMESTAMP_TOKEN_OID);

    if let Some(ref unsigned_attrs) = signer_info.unsigned_attrs {
        for attr in unsigned_attrs.iter() {
            if attr.oid == ts_oid {
                return true;
            }
        }
    }

    false
}

/// Check if a CMS SignedData contains revocation info (CRLs).
pub fn has_revocation_info(pkcs7_bytes: &[u8]) -> (bool, bool) {
    use cms::content_info::ContentInfo;
    use cms::signed_data::SignedData;

    let ci = match ContentInfo::from_der(pkcs7_bytes) {
        Ok(ci) => ci,
        Err(_) => return (false, false),
    };

    let sd = match ci.content.decode_as::<SignedData>() {
        Ok(sd) => sd,
        Err(_) => return (false, false),
    };

    let has_crls = sd.crls.is_some();
    (has_crls, false)
}

/// Extract timestamp date string from a timestamp token (best-effort).
pub fn extract_timestamp_date(_token_bytes: &[u8]) -> Option<String> {
    // The timestamp token is a CMS ContentInfo containing TSTInfo.
    // Full parsing of TSTInfo requires walking through the nested ASN.1.
    // For now, return None — the presence of the token is the key indicator.
    // A full implementation would parse the genTime field from TSTInfo.
    None
}

// --- Raw DER parsing helpers ---

/// Parse a DER SEQUENCE, returning (full_element_bytes, content_bytes).
fn parse_der_sequence(data: &[u8]) -> std::result::Result<(&[u8], &[u8]), String> {
    if data.is_empty() || data[0] != 0x30 {
        return Err("expected SEQUENCE tag (0x30)".to_string());
    }
    let (element, content_offset) = parse_der_tag_and_length(data)?;
    Ok((element, &element[content_offset..]))
}

/// Parse a single DER element, returning (element_bytes, rest).
fn parse_der_element(data: &[u8]) -> std::result::Result<(&[u8], &[u8]), String> {
    if data.is_empty() {
        return Err("unexpected end of data".to_string());
    }
    let (element, _) = parse_der_tag_and_length(data)?;
    let element_len = element.len();
    Ok((&data[..element_len], &data[element_len..]))
}

/// Parse tag + length, returning (full_element_slice, offset_to_content).
fn parse_der_tag_and_length(data: &[u8]) -> std::result::Result<(&[u8], usize), String> {
    if data.len() < 2 {
        return Err("data too short for TLV".to_string());
    }

    let mut offset = 1; // skip tag byte

    let (content_len, len_bytes) = if data[offset] < 0x80 {
        (data[offset] as usize, 1)
    } else {
        let num_len_bytes = (data[offset] & 0x7F) as usize;
        if offset + 1 + num_len_bytes > data.len() {
            return Err("length bytes extend beyond data".to_string());
        }
        let mut len = 0usize;
        for i in 0..num_len_bytes {
            len = (len << 8) | data[offset + 1 + i] as usize;
        }
        (len, 1 + num_len_bytes)
    };

    offset += len_bytes;
    let total_len = offset + content_len;

    if total_len > data.len() {
        return Err(format!(
            "element extends beyond data ({} > {})",
            total_len,
            data.len()
        ));
    }

    Ok((&data[..total_len], offset))
}

/// Encode a DER length value.
fn encode_der_length(len: usize, output: &mut Vec<u8>) {
    if len < 0x80 {
        output.push(len as u8);
    } else if len < 0x100 {
        output.push(0x81);
        output.push(len as u8);
    } else if len < 0x10000 {
        output.push(0x82);
        output.push((len >> 8) as u8);
        output.push((len & 0xFF) as u8);
    } else {
        output.push(0x83);
        output.push((len >> 16) as u8);
        output.push(((len >> 8) & 0xFF) as u8);
        output.push((len & 0xFF) as u8);
    }
}
