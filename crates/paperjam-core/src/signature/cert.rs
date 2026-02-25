use crate::error::{PdfError, Result};
use crate::signature::types::CertificateInfo;

/// Parse certificate info from a PKCS#7 (CMS) signature blob.
///
/// Extracts the signing certificate from the CMS SignedData structure.
pub fn parse_certificate_info(pkcs7_bytes: &[u8]) -> Result<CertificateInfo> {
    use cms::content_info::ContentInfo;
    use cms::signed_data::SignedData;
    use der::Decode;

    // Parse CMS ContentInfo
    let content_info = ContentInfo::from_der(pkcs7_bytes)
        .map_err(|e| PdfError::Signature(format!("Failed to parse CMS ContentInfo: {}", e)))?;

    // Extract SignedData
    let signed_data = content_info
        .content
        .decode_as::<SignedData>()
        .map_err(|e| PdfError::Signature(format!("Failed to parse SignedData: {}", e)))?;

    // Get the first certificate
    let cert_set = signed_data
        .certificates
        .ok_or_else(|| PdfError::Signature("No certificates in SignedData".to_string()))?;

    let cert_der = cert_set
        .0
        .iter()
        .next()
        .ok_or_else(|| PdfError::Signature("Certificate set is empty".to_string()))?;

    // Re-encode the certificate to DER bytes for x509-parser
    let cert_bytes = der::Encode::to_der(cert_der)
        .map_err(|e| PdfError::Signature(format!("Failed to encode cert DER: {}", e)))?;

    parse_x509_info(&cert_bytes)
}

/// Parse X.509 certificate info from DER bytes.
pub fn parse_x509_info(der_bytes: &[u8]) -> Result<CertificateInfo> {
    use x509_parser::prelude::*;

    let (_, cert) = X509Certificate::from_der(der_bytes)
        .map_err(|e| PdfError::Signature(format!("Failed to parse X.509: {}", e)))?;

    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial_number = cert.serial.to_str_radix(16);
    let not_before = cert.validity().not_before.to_rfc2822().unwrap_or_else(|_| "unknown".to_string());
    let not_after = cert.validity().not_after.to_rfc2822().unwrap_or_else(|_| "unknown".to_string());

    let is_self_signed = cert.subject() == cert.issuer();

    Ok(CertificateInfo {
        subject,
        issuer,
        serial_number,
        not_before,
        not_after,
        is_self_signed,
    })
}

/// Check if a certificate is currently valid (date check only).
pub fn is_certificate_date_valid(der_bytes: &[u8]) -> Result<bool> {
    use x509_parser::prelude::*;

    let (_, cert) = X509Certificate::from_der(der_bytes)
        .map_err(|e| PdfError::Signature(format!("Failed to parse X.509: {}", e)))?;

    Ok(cert.validity().is_valid())
}
