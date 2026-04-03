/// Information about a digital signature found in the PDF.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    /// Field name of the signature.
    pub name: String,
    /// Signer's name from the certificate (if available).
    pub signer: Option<String>,
    /// Reason for signing (if provided).
    pub reason: Option<String>,
    /// Location of signing (if provided).
    pub location: Option<String>,
    /// Signing date string from the signature dictionary.
    pub date: Option<String>,
    /// Contact info (if provided).
    pub contact_info: Option<String>,
    /// The byte range covered by this signature [offset1, len1, offset2, len2].
    pub byte_range: Option<[i64; 4]>,
    /// Certificate information (if parseable).
    pub certificate: Option<CertificateInfo>,
    /// Whether the signature covers the whole document.
    pub covers_whole_document: bool,
    /// Whether a timestamp token is present.
    pub has_timestamp: bool,
    /// Timestamp date (from the TSA), if available.
    pub timestamp_date: Option<String>,
    /// Whether OCSP responses are embedded.
    pub has_ocsp: bool,
    /// Whether CRLs are embedded.
    pub has_crls: bool,
}

/// Basic X.509 certificate information.
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// Subject common name.
    pub subject: String,
    /// Issuer common name.
    pub issuer: String,
    /// Serial number as hex string.
    pub serial_number: String,
    /// Validity start (not before) as string.
    pub not_before: String,
    /// Validity end (not after) as string.
    pub not_after: String,
    /// Whether the certificate is self-signed.
    pub is_self_signed: bool,
}

/// Result of signature verification.
#[derive(Debug, Clone)]
pub struct SignatureValidity {
    /// The signature field name.
    pub name: String,
    /// Whether the hash of signed bytes matches the signature.
    pub integrity_ok: bool,
    /// Whether the certificate is currently valid (dates check).
    pub certificate_valid: bool,
    /// Human-readable status message.
    pub message: String,
    /// Signer name (from certificate).
    pub signer: Option<String>,
    /// Whether the timestamp token is valid.
    pub timestamp_valid: Option<bool>,
    /// Whether revocation info was found and valid.
    pub revocation_ok: Option<bool>,
    /// Whether this signature has LTV information.
    pub is_ltv: bool,
}

/// Options for signing a document.
#[derive(Debug, Clone)]
pub struct SignOptions {
    /// Reason for signing.
    pub reason: Option<String>,
    /// Location of signing.
    pub location: Option<String>,
    /// Contact information.
    pub contact_info: Option<String>,
    /// Field name for the signature (default: "Signature1").
    pub field_name: String,
    /// LTV options for long-term validation.
    pub ltv: Option<LtvOptions>,
}

impl Default for SignOptions {
    fn default() -> Self {
        Self {
            reason: None,
            location: None,
            contact_info: None,
            field_name: "Signature1".to_string(),
            ltv: None,
        }
    }
}

/// Configuration for LTV (Long-Term Validation) signatures.
#[derive(Debug, Clone, Default)]
pub struct LtvOptions {
    /// TSA server URL for RFC 3161 timestamps.
    pub tsa_url: Option<String>,
    /// Pre-fetched RFC 3161 timestamp token (DER-encoded TimeStampToken).
    pub timestamp_token: Option<Vec<u8>>,
    /// OCSP response bytes (DER-encoded) to embed for revocation checking.
    pub ocsp_responses: Vec<Vec<u8>>,
    /// CRL bytes (DER-encoded) to embed for revocation checking.
    pub crls: Vec<Vec<u8>>,
}
