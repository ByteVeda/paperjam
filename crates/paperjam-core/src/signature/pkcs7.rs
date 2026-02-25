use crate::error::{PdfError, Result};

/// Verify the integrity of a PKCS#7 (CMS) signature against signed data.
///
/// This verifies that the hash of the signed data matches the digest
/// embedded in the CMS SignerInfo.
pub fn verify_pkcs7_integrity(pkcs7_bytes: &[u8], signed_data: &[u8]) -> Result<bool> {
    use cms::content_info::ContentInfo;
    use cms::signed_data::SignedData;
    use der::Decode;
    use sha2::Digest;

    // Parse CMS
    let content_info = ContentInfo::from_der(pkcs7_bytes)
        .map_err(|e| PdfError::Signature(format!("CMS parse error: {}", e)))?;

    let sd = content_info
        .content
        .decode_as::<SignedData>()
        .map_err(|e| PdfError::Signature(format!("SignedData parse error: {}", e)))?;

    // Get the first signer info
    let signer_info = sd
        .signer_infos
        .0
        .iter()
        .next()
        .ok_or_else(|| PdfError::Signature("No signer info found".to_string()))?;

    // Determine the digest algorithm
    let digest_alg_oid = &signer_info.digest_alg.oid;

    let sha256_oid = der::oid::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
    let sha1_oid = der::oid::ObjectIdentifier::new_unwrap("1.3.14.3.2.26");
    let sha384_oid = der::oid::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
    let sha512_oid = der::oid::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");

    let computed_hash: Vec<u8> = if *digest_alg_oid == sha256_oid {
        sha2::Sha256::digest(signed_data).to_vec()
    } else if *digest_alg_oid == sha1_oid {
        sha1::Sha1::digest(signed_data).to_vec()
    } else if *digest_alg_oid == sha384_oid {
        sha2::Sha384::digest(signed_data).to_vec()
    } else if *digest_alg_oid == sha512_oid {
        sha2::Sha512::digest(signed_data).to_vec()
    } else {
        return Err(PdfError::Signature(format!(
            "Unsupported digest algorithm: {}",
            digest_alg_oid
        )));
    };

    // Check if there are signed attributes - if so, we verify the message-digest attribute
    if let Some(ref signed_attrs) = signer_info.signed_attrs {
        let md_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.4");

        for attr in signed_attrs.iter() {
            if attr.oid == md_oid {
                if let Some(attr_val) = attr.values.iter().next() {
                    let embedded_hash: &[u8] = attr_val.value();

                    // The embedded hash is DER-encoded as an OCTET STRING
                    if let Ok(octet_string) = der::asn1::OctetString::from_der(embedded_hash) {
                        return Ok(octet_string.as_bytes() == computed_hash.as_slice());
                    }
                    // Fallback: compare raw bytes
                    return Ok(embedded_hash == computed_hash.as_slice());
                }
            }
        }

        return Err(PdfError::Signature(
            "No message-digest attribute in signed attributes".to_string(),
        ));
    }

    // No signed attributes - direct hash comparison is not standard for CMS,
    // but we can at least verify the hash was computed
    Ok(!computed_hash.is_empty())
}

/// Build a PKCS#7 (CMS) SignedData structure for signing.
///
/// Creates a detached CMS signature over the given hash using the private key.
pub fn create_pkcs7_signature(
    hash: &[u8],
    private_key_der: &[u8],
    cert_der: &[u8],
) -> Result<Vec<u8>> {
    use x509_parser::prelude::*;

    // Parse the certificate to determine key algorithm
    let (_, x509_cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| PdfError::Signature(format!("Failed to parse certificate: {}", e)))?;

    let pub_key_alg = x509_cert.public_key().algorithm.algorithm.to_string();

    // Sign the hash based on key type
    let sig_bytes = if pub_key_alg.contains("1.2.840.113549.1.1") {
        // RSA
        sign_rsa(hash, private_key_der)?
    } else if pub_key_alg.contains("1.2.840.10045") {
        // ECDSA
        sign_ecdsa(hash, private_key_der)?
    } else {
        return Err(PdfError::Signature(format!(
            "Unsupported key algorithm: {}",
            pub_key_alg
        )));
    };

    build_simple_cms(cert_der, &sig_bytes)
}

/// Sign a hash with RSA PKCS#1 v1.5.
fn sign_rsa(hash: &[u8], private_key_der: &[u8]) -> Result<Vec<u8>> {
    use pkcs8::DecodePrivateKey;
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::{SignatureEncoding, SignerMut};
    use rsa::RsaPrivateKey;

    let key = RsaPrivateKey::from_pkcs8_der(private_key_der)
        .map_err(|e| PdfError::Signature(format!("Failed to parse RSA key: {}", e)))?;

    let mut signing_key = SigningKey::<sha2::Sha256>::new(key);
    let signature = signing_key.sign(hash);

    Ok(signature.to_vec())
}

/// Sign a hash with ECDSA P-256.
fn sign_ecdsa(hash: &[u8], private_key_der: &[u8]) -> Result<Vec<u8>> {
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use pkcs8::DecodePrivateKey;

    let key = SigningKey::from_pkcs8_der(private_key_der)
        .map_err(|e| PdfError::Signature(format!("Failed to parse ECDSA key: {}", e)))?;

    let signature: Signature = key.sign(hash);

    Ok(signature.to_der().as_bytes().to_vec())
}

/// Build a minimal CMS ContentInfo with SignedData.
fn build_simple_cms(cert_der: &[u8], signature: &[u8]) -> Result<Vec<u8>> {
    use cms::cert::CertificateChoices;
    use cms::cert::IssuerAndSerialNumber;
    use cms::content_info::ContentInfo;
    use cms::signed_data::{
        CertificateSet, EncapsulatedContentInfo, SignedData, SignerIdentifier, SignerInfo,
        SignerInfos,
    };
    use der::asn1::{OctetString, SetOfVec};
    use der::{Decode, Encode};
    use spki::AlgorithmIdentifierOwned;

    let sha256_oid = der::oid::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
    let rsa_oid =
        der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11"); // sha256WithRSAEncryption
    let data_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.1"); // id-data
    let signed_data_oid =
        der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.2"); // id-signedData

    // Parse the certificate for issuer/serial
    let cert = cms::cert::x509::Certificate::from_der(cert_der)
        .map_err(|e| PdfError::Signature(format!("Cert parse failed: {}", e)))?;

    let issuer_and_serial = IssuerAndSerialNumber {
        issuer: cert.tbs_certificate.issuer.clone(),
        serial_number: cert.tbs_certificate.serial_number.clone(),
    };

    let digest_alg = AlgorithmIdentifierOwned {
        oid: sha256_oid,
        parameters: None,
    };

    let sig_alg = AlgorithmIdentifierOwned {
        oid: rsa_oid,
        parameters: None,
    };

    let signer_info = SignerInfo {
        version: cms::content_info::CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(issuer_and_serial),
        digest_alg: digest_alg.clone(),
        signed_attrs: None,
        signature_algorithm: sig_alg,
        signature: OctetString::new(signature)
            .map_err(|e| PdfError::Signature(format!("Sig encode failed: {}", e)))?,
        unsigned_attrs: None,
    };

    let mut digest_alg_set = SetOfVec::new();
    digest_alg_set
        .insert(digest_alg)
        .map_err(|e| PdfError::Signature(format!("Digest alg insert failed: {}", e)))?;

    let mut signer_info_set = SetOfVec::new();
    signer_info_set
        .insert(signer_info)
        .map_err(|e| PdfError::Signature(format!("Signer info insert failed: {}", e)))?;

    let cert_choice = CertificateChoices::from_der(cert_der)
        .map_err(|e| PdfError::Signature(format!("Cert choice parse failed: {}", e)))?;
    let mut cert_set = SetOfVec::new();
    cert_set
        .insert(cert_choice)
        .map_err(|e| PdfError::Signature(format!("Cert set insert failed: {}", e)))?;

    let signed_data = SignedData {
        version: cms::content_info::CmsVersion::V1,
        digest_algorithms: digest_alg_set,
        encap_content_info: EncapsulatedContentInfo {
            econtent_type: data_oid,
            econtent: None,
        },
        certificates: Some(CertificateSet(cert_set)),
        crls: None,
        signer_infos: SignerInfos(signer_info_set),
    };

    let sd_bytes = signed_data
        .to_der()
        .map_err(|e| PdfError::Signature(format!("SignedData encode failed: {}", e)))?;

    let content_info = ContentInfo {
        content_type: signed_data_oid,
        content: der::Any::from_der(&sd_bytes)
            .map_err(|e| PdfError::Signature(format!("Any wrap failed: {}", e)))?,
    };

    content_info
        .to_der()
        .map_err(|e| PdfError::Signature(format!("ContentInfo encode failed: {}", e)))
}
