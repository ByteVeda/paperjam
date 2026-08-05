/// Information about a digital signature in the document.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub name: String,
    pub signer: Option<String>,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub date: Option<String>,
    pub contact_info: Option<String>,
    pub byte_range: Option<[i64; 4]>,
    pub certificate: Option<CertificateInfo>,
    pub covers_whole_document: bool,
    pub has_timestamp: bool,
    pub timestamp_date: Option<String>,
    pub has_ocsp: bool,
    pub has_crls: bool,
}

/// X.509 certificate information.
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: String,
    pub not_after: String,
    pub is_self_signed: bool,
}

/// Result of signature verification.
#[derive(Debug, Clone)]
pub struct SignatureValidity {
    pub name: String,
    pub integrity_ok: bool,
    pub certificate_valid: bool,
    pub message: String,
    pub signer: Option<String>,
    pub timestamp_valid: Option<bool>,
    pub revocation_ok: Option<bool>,
    pub is_ltv: bool,
}

/// Options for digitally signing a PDF.
#[derive(Debug, Clone)]
pub struct SignOptions {
    pub reason: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub field_name: String,
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

/// Options for long-term validation (LTV).
#[derive(Debug, Clone, Default)]
pub struct LtvOptions {
    pub tsa_url: Option<String>,
    pub timestamp_token: Option<Vec<u8>>,
    pub ocsp_responses: Vec<Vec<u8>>,
    pub crls: Vec<Vec<u8>>,
}
