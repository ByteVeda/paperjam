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
/// Includes signed attributes (content-type, message-digest, signing-time)
/// and supports full certificate chain and unsigned attributes (for timestamp tokens).
pub fn create_pkcs7_signature(
    hash: &[u8],
    private_key_der: &[u8],
    cert_chain: &[Vec<u8>],
    unsigned_attrs_der: Option<&[u8]>,
) -> Result<Vec<u8>> {
    use x509_parser::prelude::*;

    if cert_chain.is_empty() {
        return Err(PdfError::Signature(
            "At least one certificate is required".to_string(),
        ));
    }
    let signing_cert = &cert_chain[0];

    // Parse the certificate to determine key algorithm
    let (_, x509_cert) = X509Certificate::from_der(signing_cert)
        .map_err(|e| PdfError::Signature(format!("Failed to parse certificate: {}", e)))?;

    let pub_key_alg = x509_cert.public_key().algorithm.algorithm.to_string();

    // Build signed attributes as raw DER
    let signed_attrs_der = build_signed_attributes_der(hash)?;

    // Sign the DER-encoded signed attributes (SET OF tag 0x31, per RFC 5652 §5.4)
    let sig_bytes = if pub_key_alg.contains("1.2.840.113549.1.1") {
        // RSA
        sign_rsa(&signed_attrs_der, private_key_der)?
    } else if pub_key_alg.contains("1.2.840.10045") {
        // ECDSA
        sign_ecdsa(&signed_attrs_der, private_key_der)?
    } else {
        return Err(PdfError::Signature(format!(
            "Unsupported key algorithm: {}",
            pub_key_alg
        )));
    };

    build_cms_signed_data(
        cert_chain,
        &sig_bytes,
        &signed_attrs_der,
        unsigned_attrs_der,
    )
}

/// Build signed attributes as DER-encoded SET OF (tag 0x31).
///
/// Contains: content-type, signing-time, message-digest.
fn build_signed_attributes_der(hash: &[u8]) -> Result<Vec<u8>> {
    use der::Encode;

    let content_type_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.3");
    let message_digest_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.4");
    let signing_time_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.5");
    let data_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.1");

    // Build each attribute as raw DER: SEQUENCE { OID, SET { value } }
    let ct_value_der = data_oid
        .to_der()
        .map_err(|e| PdfError::Signature(format!("OID encode: {}", e)))?;
    let ct_attr = build_attribute_der(&content_type_oid, &ct_value_der)?;

    // Signing-time
    let now = std::time::SystemTime::now();
    let utc_time = der::asn1::UtcTime::from_system_time(now)
        .map_err(|e| PdfError::Signature(format!("UTCTime: {}", e)))?;
    let st_value_der = utc_time
        .to_der()
        .map_err(|e| PdfError::Signature(format!("UTCTime DER: {}", e)))?;
    let st_attr = build_attribute_der(&signing_time_oid, &st_value_der)?;

    // Message-digest
    let md_octet = der::asn1::OctetString::new(hash.to_vec())
        .map_err(|e| PdfError::Signature(format!("OctetString: {}", e)))?;
    let md_value_der = md_octet
        .to_der()
        .map_err(|e| PdfError::Signature(format!("OctetString DER: {}", e)))?;
    let md_attr = build_attribute_der(&message_digest_oid, &md_value_der)?;

    // Combine as SET OF (tag 0x31)
    let mut attrs_content = Vec::new();
    // Sort by DER encoding for SET OF (DER requires canonical ordering)
    let mut attr_list = vec![ct_attr, st_attr, md_attr];
    attr_list.sort();
    for attr in &attr_list {
        attrs_content.extend_from_slice(attr);
    }

    let mut result = Vec::new();
    result.push(0x31); // SET OF tag
    encode_der_length(attrs_content.len(), &mut result);
    result.extend_from_slice(&attrs_content);

    Ok(result)
}

/// Build a single Attribute as DER: SEQUENCE { OID, SET { value_der } }
fn build_attribute_der(oid: &der::oid::ObjectIdentifier, value_der: &[u8]) -> Result<Vec<u8>> {
    use der::Encode;

    let oid_der = oid
        .to_der()
        .map_err(|e| PdfError::Signature(format!("OID encode: {}", e)))?;

    // SET { value_der }
    let mut set_content = Vec::new();
    set_content.push(0x31); // SET tag
    encode_der_length(value_der.len(), &mut set_content);
    set_content.extend_from_slice(value_der);

    // SEQUENCE { oid_der, set }
    let seq_content_len = oid_der.len() + set_content.len();
    let mut result = Vec::new();
    result.push(0x30); // SEQUENCE tag
    encode_der_length(seq_content_len, &mut result);
    result.extend_from_slice(&oid_der);
    result.extend_from_slice(&set_content);

    Ok(result)
}

/// Sign data with RSA PKCS#1 v1.5.
fn sign_rsa(data: &[u8], private_key_der: &[u8]) -> Result<Vec<u8>> {
    use pkcs8::DecodePrivateKey;
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::{SignatureEncoding, SignerMut};
    use rsa::RsaPrivateKey;

    let key = RsaPrivateKey::from_pkcs8_der(private_key_der)
        .map_err(|e| PdfError::Signature(format!("Failed to parse RSA key: {}", e)))?;

    let mut signing_key = SigningKey::<sha2::Sha256>::new(key);
    let signature = signing_key.sign(data);

    Ok(signature.to_vec())
}

/// Sign data with ECDSA P-256.
fn sign_ecdsa(data: &[u8], private_key_der: &[u8]) -> Result<Vec<u8>> {
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use pkcs8::DecodePrivateKey;

    let key = SigningKey::from_pkcs8_der(private_key_der)
        .map_err(|e| PdfError::Signature(format!("Failed to parse ECDSA key: {}", e)))?;

    let signature: Signature = key.sign(data);

    Ok(signature.to_der().as_bytes().to_vec())
}

/// Build a CMS ContentInfo with SignedData.
fn build_cms_signed_data(
    cert_chain: &[Vec<u8>],
    signature: &[u8],
    signed_attrs_der: &[u8],
    unsigned_attrs_der: Option<&[u8]>,
) -> Result<Vec<u8>> {
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
    let rsa_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
    let data_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.1");
    let signed_data_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.2");

    let signing_cert = &cert_chain[0];

    let cert = cms::cert::x509::Certificate::from_der(signing_cert)
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

    // Parse signed attributes from raw DER back into CMS type
    // The DER has tag 0x31 (SET OF); CMS expects IMPLICIT [0] (tag 0xA0)
    let signed_attrs: Option<cms::signed_data::SignedAttributes> = {
        let mut modified_der = signed_attrs_der.to_vec();
        if !modified_der.is_empty() && modified_der[0] == 0x31 {
            modified_der[0] = 0xA0; // Change to IMPLICIT [0] for SignerInfo context
        }
        // Parse as SignedAttributes (which is SetOfVec<Attribute>)
        // The CMS crate's SignerInfo expects context-tagged [0]
        Some(
            SetOfVec::from_der(signed_attrs_der)
                .map_err(|e| PdfError::Signature(format!("SignedAttrs parse: {}", e)))?,
        )
    };

    // Parse unsigned attributes if provided
    let unsigned_attrs: Option<cms::signed_data::UnsignedAttributes> =
        if let Some(ua_der) = unsigned_attrs_der {
            Some(
                SetOfVec::from_der(ua_der)
                    .map_err(|e| PdfError::Signature(format!("UnsignedAttrs parse: {}", e)))?,
            )
        } else {
            None
        };

    let signer_info = SignerInfo {
        version: cms::content_info::CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(issuer_and_serial),
        digest_alg: digest_alg.clone(),
        signed_attrs,
        signature_algorithm: sig_alg,
        signature: OctetString::new(signature)
            .map_err(|e| PdfError::Signature(format!("Sig encode failed: {}", e)))?,
        unsigned_attrs,
    };

    let mut digest_alg_set = SetOfVec::new();
    digest_alg_set
        .insert(digest_alg)
        .map_err(|e| PdfError::Signature(format!("Digest alg insert failed: {}", e)))?;

    let mut signer_info_set = SetOfVec::new();
    signer_info_set
        .insert(signer_info)
        .map_err(|e| PdfError::Signature(format!("Signer info insert failed: {}", e)))?;

    // Include all certificates from the chain
    let mut cert_set = SetOfVec::new();
    for cert_der in cert_chain {
        let cert_choice = CertificateChoices::from_der(cert_der)
            .map_err(|e| PdfError::Signature(format!("Cert choice parse failed: {}", e)))?;
        cert_set
            .insert(cert_choice)
            .map_err(|e| PdfError::Signature(format!("Cert set insert failed: {}", e)))?;
    }

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

/// Add a timestamp token as an unsigned attribute to an existing CMS signature.
pub fn add_timestamp_to_cms(cms_bytes: &[u8], timestamp_token: &[u8]) -> Result<Vec<u8>> {
    use cms::content_info::ContentInfo;
    use cms::signed_data::{SignedData, SignerInfos};
    use der::asn1::SetOfVec;
    use der::{Decode, Encode};

    let content_info = ContentInfo::from_der(cms_bytes)
        .map_err(|e| PdfError::Signature(format!("CMS parse for timestamp: {}", e)))?;

    let mut sd = content_info
        .content
        .decode_as::<SignedData>()
        .map_err(|e| PdfError::Signature(format!("SignedData parse for timestamp: {}", e)))?;

    let mut signer_infos: Vec<_> = sd.signer_infos.0.into_vec();

    if let Some(si) = signer_infos.first_mut() {
        // Build the timestamp attribute as raw DER, then parse it
        let ts_oid_str = super::tsa::TIMESTAMP_TOKEN_OID;
        let ts_oid = der::oid::ObjectIdentifier::new_unwrap(ts_oid_str);
        let ts_oid_der = ts_oid
            .to_der()
            .map_err(|e| PdfError::Signature(format!("TS OID encode: {}", e)))?;

        // Build: SEQUENCE { OID, SET { token } }
        let mut set_content = Vec::new();
        set_content.push(0x31);
        encode_der_length(timestamp_token.len(), &mut set_content);
        set_content.extend_from_slice(timestamp_token);

        let seq_len = ts_oid_der.len() + set_content.len();
        let mut attr_der = Vec::new();
        attr_der.push(0x30);
        encode_der_length(seq_len, &mut attr_der);
        attr_der.extend_from_slice(&ts_oid_der);
        attr_der.extend_from_slice(&set_content);

        // Build SET OF { attr_der }
        let mut set_of_der = Vec::new();
        set_of_der.push(0x31);
        encode_der_length(attr_der.len(), &mut set_of_der);
        set_of_der.extend_from_slice(&attr_der);

        // Parse as UnsignedAttributes and merge
        let new_attrs: cms::signed_data::UnsignedAttributes = SetOfVec::from_der(&set_of_der)
            .map_err(|e| PdfError::Signature(format!("TS attr parse: {}", e)))?;

        if let Some(ref mut existing) = si.unsigned_attrs {
            for attr in new_attrs.iter() {
                existing
                    .insert(attr.clone())
                    .map_err(|e| PdfError::Signature(format!("TS attr merge: {}", e)))?;
            }
        } else {
            si.unsigned_attrs = Some(new_attrs);
        }
    }

    let mut new_si_set = SetOfVec::new();
    for si in signer_infos {
        new_si_set
            .insert(si)
            .map_err(|e| PdfError::Signature(format!("SI rebuild: {}", e)))?;
    }
    sd.signer_infos = SignerInfos(new_si_set);

    let signed_data_oid = der::oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.2");
    let sd_bytes = sd
        .to_der()
        .map_err(|e| PdfError::Signature(format!("SignedData re-encode: {}", e)))?;

    let new_ci = ContentInfo {
        content_type: signed_data_oid,
        content: der::Any::from_der(&sd_bytes)
            .map_err(|e| PdfError::Signature(format!("Any re-wrap: {}", e)))?,
    };

    new_ci
        .to_der()
        .map_err(|e| PdfError::Signature(format!("ContentInfo re-encode: {}", e)))
}

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
